//! Rain VM: A virtual machine for Rain ML.

use std::collections::HashMap;
use std::fs;
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
        let w = decode_u32(&mut iter)?;
        self.0.insert(Reg(0), w);
        Ok(())
    }

    fn get(&self, r: Reg) -> Option<u32> {
        self.0.get(&r).map(|&x| x)
    }
}

fn decode_u32<'a, T>(iter: &mut T) -> Result<u32, ExecutionError>
where
    T: Iterator<Item = &'a u8>,
{
    let mut u = 0;
    for _ in 0..4 {
        let n: u32 = (*iter.next().ok_or(ExecutionError::TruncatedU32)?).into();
        u <<= 8;
        u |= n;
    }
    Ok(u)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::discriminant;
    use test::Bencher;

    macro_rules! decode_u32_ok {
        ($x:expr, $r:expr) => {
            assert_eq!(decode_u32(&mut $x.iter()).ok(), Some($r));
        };
    }

    macro_rules! decode_u32_err {
        ($x:expr) => {
            assert_eq!(
                discriminant(&decode_u32(&mut $x.iter()).err().unwrap()),
                discriminant(&ExecutionError::TruncatedU32)
            );
        };
    }

    #[test]
    fn test_decode_u32() {
        decode_u32_ok!([0, 0, 0, 0], 0);
        decode_u32_ok!([0, 0, 0, 1], 1);
        decode_u32_ok!([0, 0, 34, 130], 8834);
        decode_u32_ok!([1, 0, 18, 1], 16781825);
        decode_u32_ok!([255; 4], 4294967295);

        decode_u32_err!([]);
        decode_u32_err!([100]);
        decode_u32_err!([20; 2]);
        decode_u32_err!([7; 3]);

        decode_u32_ok!([7; 5], 117901063);
        decode_u32_ok!([1, 2, 3, 4, 5], 16909060);
    }

    #[bench]
    fn bench_decode_u32_1(b: &mut Bencher) {
        b.iter(|| decode_u32(&mut [0, 0, 0, 0].iter()));
    }

    #[bench]
    fn bench_decode_u32_2(b: &mut Bencher) {
        b.iter(|| decode_u32(&mut [255; 4].iter()));
    }
}
