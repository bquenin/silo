//! Low-level binary read helpers, modeled on the Python parser's
//! `read_cstr`, `read_tb_str`, `read_uint32`, `read_byte`.

use std::io::Read;

use super::error::{ParseError, Result};

pub struct R<'a, T: Read> {
    pub inner: &'a mut T,
    pub offset: u64,
}

impl<'a, T: Read> R<'a, T> {
    pub fn new(inner: &'a mut T) -> Self {
        Self { inner, offset: 0 }
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                ParseError::Eof { offset: self.offset }
            } else {
                ParseError::Io(e)
            }
        })?;
        self.offset += buf.len() as u64;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut b = [0u8; 1];
        self.read_exact(&mut b)?;
        Ok(b[0])
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        let mut b = [0u8; 4];
        self.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    /// Read `length` ASCII bytes as a UTF-8 string (lossy).
    pub fn read_cstr(&mut self, length: usize) -> Result<String> {
        let mut buf = vec![0u8; length];
        self.read_exact(&mut buf)?;
        // The Python code uses lossy UTF-8 decoding. Match that.
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    /// Read a "TB string" — a UTF-16 LE sequence terminated by a NUL u16,
    /// OR exactly `length` u16 code units when `length >= 0`.
    pub fn read_tb_str(&mut self, length: Option<usize>) -> Result<String> {
        let mut out = String::new();
        loop {
            let mut b = [0u8; 2];
            self.read_exact(&mut b)?;
            let unit = u16::from_le_bytes(b);
            if length.is_none() && unit == 0 {
                break;
            }
            // Decode as UTF-16 code unit. Surrogates are rare in replay strings;
            // for simplicity treat each unit as a BMP code point. Lossy fallback
            // for replacement chars.
            if let Some(c) = char::from_u32(unit as u32) {
                out.push(c);
            } else {
                out.push(char::REPLACEMENT_CHARACTER);
            }
            if let Some(l) = length {
                if out.chars().count() >= l {
                    break;
                }
            }
        }
        Ok(out)
    }

    /// Skip exactly `n` bytes.
    pub fn skip(&mut self, n: usize) -> Result<()> {
        let mut sink = vec![0u8; n];
        self.read_exact(&mut sink)?;
        Ok(())
    }
}
