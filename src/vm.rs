//! Rain VM: A virtual machine for Rain ML.

use std::fs::File;
use std::io;
use std::io::Read;

use failure::Error;

use crate::version;

/// An execution error.
#[derive(Fail, Debug)]
pub enum ExecutionError {
    /// Missing version.
    #[fail(display = "missing version")]
    MissingVersion,

    /// Given an unknown version.
    #[fail(display = "unknown version: {}", version)]
    UnknownVersion { version: u8 },

    /// Given a mismatched version.
    #[fail(display = "version mismatch: {}", version)]
    VersionMismatch { version: u8 },

    /// File-open error.
    #[fail(display = "opening file {:?}: {}", filename, error)]
    FileOpen { filename: String, error: io::Error },
}

/// Executes a file.
pub fn execute_file(filename: &str) -> Result<u8, Error> {
    let f = File::open(filename).map_err(|e| ExecutionError::FileOpen {
        filename: filename.to_string(),
        error: e,
    })?;
    let v = f.bytes().collect::<io::Result<Vec<u8>>>()?;
    let ret = execute_bytes(v)?;
    Ok(ret)
}

/// Executes a sequence of bytes.
pub fn execute_bytes(v: Vec<u8>) -> Result<u8, ExecutionError> {
    let mut iter = v.into_iter();
    match iter.next() {
        None => Err(ExecutionError::MissingVersion),
        Some(b) => {
            if b == version::BYTE_VERSION {
                Ok(0)
            } else {
                Err(ExecutionError::VersionMismatch { version: b })
            }
        }
    }
}
