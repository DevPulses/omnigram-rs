//! tdata local keys, passcode KDF and the encrypted file container.
//!
//! Source of truth: `tdesktop/Telegram/SourceFiles/storage/details/storage_file_utilities.{h,cpp}`.
//!
//! Layers:
//! - [`local_key`] — the 256-byte local key and its two PBKDF2 derivations.
//! - [`passcode`] — the passcode KDF record and the Argon2id / scrypt wrap.
//! - [`local_cipher`] — AES-256-IGE with the local-key/`key128` derivation.
//! - [`descriptor`] — the `[uint32 size][blocks]` payload being encrypted.
//! - [`prepare`] / [`decrypt`] — `PrepareEncrypted` / `DecryptLocal`.
//! - [`container`] — the `TDF$` header, the MD5 file signature and its check.
//! - [`file_key`] — `FileKey` filenames and their `0`/`1`/`s` postfixes.
//! - [`writer`] / [`reader`] — writing and reading one such file.
//! - [`manager`] — the write queue, its coalescing and the async thread.

pub mod container;
pub mod decrypt;
pub mod descriptor;
pub mod file_key;
pub mod local_cipher;
pub mod local_key;
pub mod manager;
pub mod passcode;
pub mod prepare;
pub mod reader;
pub mod version;
pub mod writer;

pub use decrypt::Decrypted;
pub use descriptor::{DataReader, EncryptedDescriptor};
pub use file_key::{clear_key, generate_key, key_already_used, to_file_part, FileKey};
pub use local_key::{create_legacy_local_key, create_local_key, LocalKey, LOCAL_KEY_SIZE};
pub use passcode::{create_passcode_key, default_passcode_kdf, PasscodeKdf};
pub use prepare::{prepare_encrypted, prepare_encrypted_with};
pub use reader::{read_encrypted_file, read_encrypted_file_for, read_file, FileReadDescriptor};
pub use writer::FileWriteDescriptor;