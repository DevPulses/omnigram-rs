//! The write queue: coalescing, ordered removal and the async worker thread.
//!
//! Source of truth: `WriteManager`, `AsyncWriteManager` and the free
//! `ScheduleRemoveFiles` / `ScheduleRemoveFolder` / `Sync` / `Finish` helpers in
//! `storage_file_utilities.cpp`.
//!
//! Queue rules kept as they are in C++:
//! - writes coalesce per base path, but only *after* the last queued removal,
//!   so a removal never erases a later write and no earlier write outlives it;
//! - the async manager owns one worker thread that drains operations in order,
//!   and a synchronous write is performed by that same thread and awaited.
//!
//! Failures are reported through return values only: the C++ code logs them and
//! nothing observable about the file changes, so neither does this port.

use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;

use crate::container::{header_bytes, SIGNATURE_LEN};
use crate::file_key::{append_to_path, with_postfix};
use crate::version::APP_VERSION;

/// One queued file write: the blocks, their signature and where they go.
#[derive(Debug, Clone)]
pub struct WriteEntry {
    pub base_path: PathBuf,
    pub base: PathBuf,
    pub data: Vec<u8>,
    pub md5: [u8; SIGNATURE_LEN],
}

/// One queued removal of files and/or a folder.
#[derive(Debug, Clone, Default)]
pub struct RemoveEntry {
    pub files: Vec<PathBuf>,
    pub folder: Option<PathBuf>,
}

/// A queued operation.
#[derive(Debug, Clone)]
pub enum WriteOperation {
    Write(WriteEntry),
    Remove(RemoveEntry),
}

/// The queue itself, drained either by the caller or by a worker thread.
#[derive(Debug, Default)]
pub struct WriteManager {
    scheduled: VecDeque<WriteOperation>,
}

impl WriteManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `write` — queues an entry, replacing a pending one for the same record.
    pub fn write(&mut self, entry: WriteEntry) {
        match self.scheduled_index(&entry.base) {
            Some(index) => self.scheduled[index] = WriteOperation::Write(entry),
            None => self.scheduled.push_back(WriteOperation::Write(entry)),
        }
    }

    /// `writeSync` — flushes everything before the last removal, drops a pending
    /// write for the same record and writes this entry now.
    pub fn write_sync(&mut self, entry: WriteEntry) -> bool {
        self.flush_before_last_removal();
        if let Some(index) = self.scheduled_index(&entry.base) {
            self.scheduled.remove(index);
        }
        self.write_now(entry)
    }

    /// `for (auto left = segment() - begin; left > 0; --left)`. With no removal
    /// queued the segment is the end of the queue, so everything is flushed.
    fn flush_before_last_removal(&mut self) {
        let count = self
            .scheduled
            .iter()
            .rposition(|operation| matches!(operation, WriteOperation::Remove(_)))
            .unwrap_or(self.scheduled.len());
        for _ in 0..count {
            self.write_one_scheduled_now();
        }
    }

    /// `remove` — queues a removal after every pending write.
    pub fn remove(&mut self, entry: RemoveEntry) {
        self.scheduled.push_back(WriteOperation::Remove(entry));
    }

    /// `writeSyncAll` — drains the whole queue.
    pub fn write_sync_all(&mut self) {
        while self.write_one_scheduled_now() {}
    }

    /// Whether anything is still queued.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scheduled.is_empty()
    }

    fn write_one_scheduled_now(&mut self) -> bool {
        let Some(operation) = self.scheduled.pop_front() else {
            return false;
        };
        match operation {
            WriteOperation::Write(entry) => {
                self.write_now(entry);
            }
            WriteOperation::Remove(entry) => remove_now(&entry),
        }
        true
    }

    /// Index of the first queued write for `base`, searched after the last
    /// removal so coalescing never crosses one.
    fn scheduled_index(&self, base: &Path) -> Option<usize> {
        let from = self.segment();
        self.scheduled
            .iter()
            .skip(from)
            .position(|operation| match operation {
                WriteOperation::Write(entry) => entry.base == base,
                WriteOperation::Remove(_) => false,
            })
            .map(|offset| from + offset)
    }

    /// Index right after the last removal in the queue.
    fn segment(&self) -> usize {
        self.scheduled
            .iter()
            .rposition(|operation| matches!(operation, WriteOperation::Remove(_)))
            .map_or(0, |index| index + 1)
    }

    /// `writeNow` — writes through a temporary file, falling back to the plain
    /// name and a rename-over of the current one.
    fn write_now(&mut self, entry: WriteEntry) -> bool {
        let safe = with_postfix(&entry.base, 's');
        let simple = with_postfix(&entry.base, '0');
        let backup = with_postfix(&entry.base, '1');

        if let Some(temp) = write_atomically(&safe, &entry) {
            let _ = fs::remove_file(&simple);
            let _ = fs::remove_file(&backup);
            return temp;
        }

        if write_plain(&simple, &entry).is_ok() {
            let _ = fs::remove_file(&backup);
            if rename_over(&simple, &safe).is_ok() {
                return true;
            }
            let _ = fs::remove_file(&safe);
        }
        false
    }
}

/// Writes `entry` into a temporary file and renames it over `target`.
///
/// Returns `None` when the temporary file could not be produced at all, which
/// is the signal to fall back to the plain path.
fn write_plain(target: &Path, entry: &WriteEntry) -> std::io::Result<()> {
    let mut file = create_with_header(target, entry)?;
    file.write_all(&entry.data)?;
    file.write_all(&entry.md5)?;
    file.sync_all()?;
    Ok(())
}

/// Writes `entry` into a sibling temporary file, then renames it over `target`.
fn write_atomically(target: &Path, entry: &WriteEntry) -> Option<bool> {
    let temp = append_to_path(target, ".tmp");
    let written = write_plain(&temp, entry);
    if written.is_err() {
        let _ = fs::remove_file(&temp);
        return None;
    }
    if rename_over(&temp, target).is_err() {
        let _ = fs::remove_file(&temp);
        return None;
    }
    Some(true)
}

/// Opens `path` for writing and puts the container header in front.
///
/// A missing directory is created once, exactly like `writeHeader` retries the
/// open after `mkpath`.
fn create_with_header(
    path: &Path,
    entry: &WriteEntry,
) -> std::io::Result<fs::File> {
    let mut file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => {
            if path.exists() {
                return Err(error);
            }
            fs::create_dir_all(entry.base_path.clone())?;
            fs::File::create(path)?
        }
    };
    file.write_all(&header_bytes(APP_VERSION))?;
    Ok(file)
}

/// Renames over an existing target.
fn rename_over(from: &Path, to: &Path) -> std::io::Result<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(error) => {
            // Windows refuses to replace; the C++ helper removes first as well.
            if to.exists() {
                fs::remove_file(to)?;
                fs::rename(from, to)
            } else {
                Err(error)
            }
        }
    }
}

/// `removeNow` — deletes every listed file and the optional folder.
fn remove_now(entry: &RemoveEntry) {
    for file in &entry.files {
        let _ = fs::remove_file(file);
    }
    if let Some(folder) = &entry.folder {
        let _ = fs::remove_dir_all(folder);
    }
}

/// Messages the worker thread understands.
enum Message {
    /// Fire-and-forget operation, drained in order.
    Operation(WriteOperation),
    /// Write now and report the result.
    WriteSync(WriteEntry, mpsc::Sender<bool>),
    /// Drain everything and acknowledge.
    Sync(mpsc::Sender<()>),
    /// Drain everything, acknowledge and stop the thread.
    Stop(mpsc::Sender<()>),
}

/// `AsyncWriteManager` — the queue behind a lazily started worker thread.
#[derive(Debug, Default)]
pub struct AsyncWriteManager {
    sender: Option<mpsc::Sender<Message>>,
    finished: bool,
}

impl AsyncWriteManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn sender(&mut self) -> &mpsc::Sender<Message> {
        assert!(!self.finished, "the write manager has already finished");
        self.sender.get_or_insert_with(|| {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || serve(receiver));
            sender
        })
    }

    /// `write` — queues a write; the worker performs it as soon as it can.
    pub fn write(&mut self, entry: WriteEntry) {
        let _ = self
            .sender()
            .send(Message::Operation(WriteOperation::Write(entry)));
    }

    /// `writeSync` — the worker writes this entry now, and the result is
    /// returned to the caller.
    pub fn write_sync(&mut self, entry: WriteEntry) -> bool {
        let (ack, result) = mpsc::channel();
        if self.sender().send(Message::WriteSync(entry, ack)).is_err() {
            return false;
        }
        result.recv().unwrap_or(false)
    }

    /// `remove` — queues a removal.
    pub fn remove(&mut self, entry: RemoveEntry) {
        let _ = self
            .sender()
            .send(Message::Operation(WriteOperation::Remove(entry)));
    }

    /// `sync` — waits until everything queued so far is on disk.
    pub fn sync(&mut self) {
        if self.sender.is_none() {
            return;
        }
        let (ack, done) = mpsc::channel();
        if self.sender().send(Message::Sync(ack)).is_ok() {
            let _ = done.recv();
        }
    }

    /// `stop` — drains the queue, stops the thread and refuses later writes.
    pub fn stop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let (ack, done) = mpsc::channel();
            if sender.send(Message::Stop(ack)).is_ok() {
                let _ = done.recv();
            }
        }
        self.finished = true;
    }
}

/// The worker loop: one queue, drained in order.
fn serve(receiver: mpsc::Receiver<Message>) {
    let mut manager = WriteManager::new();
    while let Ok(message) = receiver.recv() {
        match message {
            Message::Operation(operation) => {
                manager.scheduled.push_back(operation);
                manager.write_sync_all();
            }
            Message::WriteSync(entry, ack) => {
                let result = manager.write_sync(entry);
                let _ = ack.send(result);
            }
            Message::Sync(ack) => {
                manager.write_sync_all();
                let _ = ack.send(());
            }
            Message::Stop(ack) => {
                manager.write_sync_all();
                let _ = ack.send(());
                break;
            }
        }
    }
}

static MANAGER: OnceLock<Mutex<AsyncWriteManager>> = OnceLock::new();

/// The process-wide write manager.
#[must_use]
pub fn manager() -> &'static Mutex<AsyncWriteManager> {
    MANAGER.get_or_init(|| Mutex::new(AsyncWriteManager::new()))
}

fn lock() -> std::sync::MutexGuard<'static, AsyncWriteManager> {
    manager().lock().unwrap_or_else(|error| error.into_inner())
}

/// Queues a write on the process-wide manager.
pub fn schedule_write(entry: WriteEntry) {
    lock().write(entry);
}

/// Performs a write synchronously on the process-wide manager.
#[must_use]
pub fn write_sync(entry: WriteEntry) -> bool {
    lock().write_sync(entry)
}

/// `ScheduleRemoveFiles` — queues a batch of files for removal.
pub fn schedule_remove_files(files: Vec<PathBuf>) {
    if !files.is_empty() {
        lock().remove(RemoveEntry {
            files,
            folder: None,
        });
    }
}

/// `ScheduleRemoveFolder` — queues a folder for recursive removal.
pub fn schedule_remove_folder(folder: PathBuf) {
    assert!(!folder.as_os_str().is_empty(), "an empty folder path");
    lock().remove(RemoveEntry {
        files: Vec::new(),
        folder: Some(folder),
    });
}

/// `Sync` — waits for everything queued so far.
pub fn sync() {
    lock().sync();
}

/// `Finish` — drains the queue and stops the worker for good.
pub fn finish() {
    lock().stop();
}