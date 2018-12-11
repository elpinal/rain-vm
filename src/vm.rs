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

    /// Given an unknown opcode.
    #[fail(display = "no such instruction: {}", opcode)]
    NoSuchInstruction { opcode: u8 },

    /// No such register.
    #[fail(display = "no such register: {:?}", reg)]
    NoSuchRegister { reg: Reg },
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
    f.get(&Reg(0)).ok_or(ExecutionError::NoResult)
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Reg(u8);

struct File(HashMap<Reg, u32>);

// Shifts 3 bits.
const SHIFT_OPCODE: u8 = 3;

const OPCODE_MOVE: u8 = 0;
const OPCODE_HALT: u8 = 1;
const OPCODE_ADD: u8 = 2;

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
        loop {
            let b = *must_next(&mut iter)?;
            match b >> SHIFT_OPCODE {
                OPCODE_MOVE => {
                    // Move.
                    if b & 0b100 == 0 {
                        unimplemented!(); // Register to register.
                    } else {
                        self.mov_imm(&mut iter)?;
                    }
                }
                OPCODE_HALT => return Ok(()),
                OPCODE_ADD => {
                    // Add.
                    let bits = b & 0b11;
                    if b & 0b100 == 0 {
                        unimplemented!(); // Register-register operation.
                    } else {
                        self.add_imm(&mut iter, bits)?;
                    }
                }
                b => return Err(ExecutionError::NoSuchInstruction { opcode: b }),
            }
        }
    }

    fn get(&self, r: &Reg) -> Option<u32> {
        self.0.get(r).cloned()
    }

    fn insert(&mut self, r: Reg, w: u32) {
        self.0.insert(r, w);
    }

    /// "Move immediate" instruction.
    fn mov_imm<'a, T>(&mut self, iter: &mut T) -> Result<(), ExecutionError>
    where
        T: Iterator<Item = &'a u8>,
    {
        let b = must_next(iter)? & 0b11111;
        let r = Reg(b);
        let w = decode_u32(iter)?;
        self.insert(r, w);
        Ok(())
    }

    /// "Add immediate" instruction.
    /// The parameter `extra_bits` is assumed to be a two-bit integer.
    /// Arithmetic overflow is ignored.
    fn add_imm<'a, T>(&mut self, iter: &mut T, extra_bits: u8) -> Result<(), ExecutionError>
    where
        T: Iterator<Item = &'a u8>,
    {
        let b = must_next(iter)?;
        let lower = b >> 5;
        let upper = extra_bits << 3;
        let src = Reg(lower | upper);

        let w = decode_u32(iter)?;
        let v = self
            .get(&src)
            .ok_or(ExecutionError::NoSuchRegister { reg: src })?;
        self.insert(Reg(b & 0b11111), v + w);
        Ok(())
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

fn must_next<'a, T>(iter: &mut T) -> Result<&'a u8, ExecutionError>
where
    T: Iterator<Item = &'a u8>,
{
    iter.next().ok_or(ExecutionError::UnexpectedEndOfProgram)
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
