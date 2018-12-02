//! Rain VM: A virtual machine for Rain ML.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Read;
use std::slice;

use failure::Error;

use crate::version;

/// An execution error.
#[derive(Fail, Debug)]
pub enum ExecutionError {
    /// Missing version.
    #[fail(display = "missing version")]
    MissingVersion,

    /// Given a mismatched version.
    #[fail(display = "version mismatch: {}", version)]
    VersionMismatch { version: u8 },

    /// File-open error.
    #[fail(display = "opening file {:?}: {}", filename, error)]
    FileOpen { filename: String, error: io::Error },

    /// Reached the unexpected end of program.
    #[fail(display = "unexpected end of program")]
    UnexpectedEndOfProgram,

    /// No result in the result register, `R0`.
    #[fail(display = "no result in the result register")]
    NoResult,

    /// 32-bit integers should have 4 bytes, but there are fewer bytes.
    #[fail(display = "truncated 32-bit integer")]
    TruncatedU32,
}

/// Executes a file.
pub fn execute_file(filename: &str) -> Result<u32, Error> {
    let f = fs::File::open(filename).map_err(|e| ExecutionError::FileOpen {
        filename: filename.to_string(),
        error: e,
    })?;
    let v = f.bytes().collect::<io::Result<Vec<u8>>>()?;
    let ret = execute_bytes(v)?;
    Ok(ret)
}

/// Executes a sequence of bytes.
pub fn execute_bytes(v: Vec<u8>) -> Result<u32, ExecutionError> {
    let mut f = File(HashMap::new());
    f.execute_bytes(v)?;
    f.get(Reg(0)).ok_or(ExecutionError::NoResult)
}

#[derive(PartialEq, Eq, Hash)]
struct Reg(u8);

struct File(HashMap<Reg, u32>);

impl File {
    /// Executes a sequence of bytes.
    pub fn execute_bytes(&mut self, v: Vec<u8>) -> Result<(), ExecutionError> {
        let mut iter = v.iter();
        match iter.next() {
            None => return Err(ExecutionError::MissingVersion),
            Some(&b) => {
                if b != version::BYTE_VERSION {
                    return Err(ExecutionError::VersionMismatch { version: b });
                }
            }
        }
        match iter.next() {
            None => Err(ExecutionError::UnexpectedEndOfProgram),
            Some(&b) => {
                self.0.insert(Reg(0), b.into());
                Ok(())
            }
        }
    }

    fn get(&self, r: Reg) -> Option<u32> {
        self.0.get(&r).map(|&x| x)
    }
}

fn decode_u32(mut iter: slice::Iter<u8>) -> Result<u32, ExecutionError> {
    let mut u: u32 = 0;
    for _ in 0..4 {
        let n: u32 = (*iter.next().ok_or(ExecutionError::TruncatedU32)?).into();
        u <<= 8;
        u += n;
    }
    Ok(u)
}
