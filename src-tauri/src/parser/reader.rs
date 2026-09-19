//! Low-level binary read helpers, modeled on the Python parser's
//! `read_cstr`, `read_tb_str`, `read_uint32`, `read_byte`.

use std::io::Read;

use super::error::{ParseError, Result};

const MAX_STRING_BYTES: usize = 1024 * 1024;
const MAX_UTF16_UNITS: usize = 64 * 1024;

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
                ParseError::Eof {
                    offset: self.offset,
                }
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
        if length > MAX_STRING_BYTES {
            return Err(ParseError::LimitExceeded {
                field: "string",
                length,
                limit: MAX_STRING_BYTES,
            });
        }
        let mut buf = vec![0u8; length];
        self.read_exact(&mut buf)?;
        // The Python code uses lossy UTF-8 decoding. Match that.
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    /// Read a "TB string" — a UTF-16 LE sequence terminated by a NUL u16,
    /// OR exactly `length` u16 code units when `length >= 0`.
    pub fn read_tb_str(&mut self, length: Option<usize>) -> Result<String> {
        if length.is_some_and(|n| n > MAX_UTF16_UNITS) {
            return Err(ParseError::LimitExceeded {
                field: "UTF-16 string",
                length: length.unwrap(),
                limit: MAX_UTF16_UNITS,
            });
        }
        let mut units = Vec::new();
        while length != Some(units.len()) {
            let mut b = [0u8; 2];
            self.read_exact(&mut b)?;
            let unit = u16::from_le_bytes(b);
            if length.is_none() && unit == 0 {
                break;
            }
            if units.len() == MAX_UTF16_UNITS {
                return Err(ParseError::LimitExceeded {
                    field: "UTF-16 string",
                    length: units.len() + 1,
                    limit: MAX_UTF16_UNITS,
                });
            }
            units.push(unit);
        }
        Ok(String::from_utf16_lossy(&units))
    }

    /// Skip exactly `n` bytes.
    pub fn skip(&mut self, n: usize) -> Result<()> {
        let mut sink = [0u8; 4096];
        let mut remaining = n;
        while remaining > 0 {
            let count = remaining.min(sink.len());
            self.read_exact(&mut sink[..count])?;
            remaining -= count;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_lengths_are_rejected_before_reading_or_allocating() {
        let mut bytes = std::io::Cursor::new([]);
        let mut reader = R::new(&mut bytes);
        assert!(matches!(
            reader.read_cstr(MAX_STRING_BYTES + 1),
            Err(ParseError::LimitExceeded { .. })
        ));
        assert!(matches!(
            reader.read_tb_str(Some(MAX_UTF16_UNITS + 1)),
            Err(ParseError::LimitExceeded { .. })
        ));
        assert_eq!(reader.offset, 0);
    }

    #[test]
    fn utf16_lengths_count_code_units_and_zero_consumes_nothing() {
        let text = "A\u{1f680}";
        let mut bytes = std::io::Cursor::new(
            text.encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        let mut reader = R::new(&mut bytes);
        assert_eq!(reader.read_tb_str(Some(0)).unwrap(), "");
        assert_eq!(reader.offset, 0);
        assert_eq!(reader.read_tb_str(Some(3)).unwrap(), text);
        assert_eq!(reader.offset, 6);
    }
}
