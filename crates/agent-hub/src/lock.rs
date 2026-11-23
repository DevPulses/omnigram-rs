//! Файловая блокировка состояния через OS flock.
//!
//! Состояние хранится в `.agent-state.json`. Любая операция (claim/release/
//! complete/next) захватывает exclusive flock на файле перед чтением и держит
//! его до атомарной записи результата, чтобы параллельные агенты не затирали
//! изменения друг друга.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;

pub struct StateGuard {
    file: File,
}

impl StateGuard {
    /// Открыть файл состояния и захватить exclusive flock.
    /// Файл создаётся, если его ещё нет.
    pub fn acquire(path: &std::path::Path) -> io::Result<StateGuard> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(StateGuard { file })
    }

    /// Прочитать содержимое файла, пока блокировка удерживается.
    pub fn read(&mut self) -> io::Result<String> {
        use std::io::Read;
        let mut buf = String::new();
        self.file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    /// Атомарно заменить содержимое: write-to-tmp + rename.
    pub fn write_atomic(&self, path: &std::path::Path, content: &str) -> io::Result<()> {
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

impl Drop for StateGuard {
    fn drop(&mut self) {
        unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}
