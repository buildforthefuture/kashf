// Copyright (c) <OWNER>

// Bounds-checked byte reader. Every read validates bounds and returns a typed
// Result; malformed or truncated input never panics.

use crate::error::{Error, Result};

/// Reads primitive values from a byte slice. Tracks a cursor and rejects any
/// access past the end of the buffer.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn bytes(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Advance the cursor by `n` bytes, bounds-checked.
    pub fn advance(&mut self, n: usize) -> Result<()> {
        self.ensure(n)?;
        self.pos = self.pos.checked_add(n).ok_or_else(|| Error::Overflow {
            operation: "reader advance".to_string(),
        })?;
        Ok(())
    }

    /// Seek to an absolute offset, bounds-checked.
    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.data.len() {
            return Err(Error::OutOfBounds {
                offset: pos,
                len: 0,
                max: self.data.len(),
            });
        }
        self.pos = pos;
        Ok(())
    }

    /// Skip `n` bytes without reading them.
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.advance(n)
    }

    fn ensure(&self, n: usize) -> Result<()> {
        let end = self.pos.checked_add(n).ok_or_else(|| Error::Overflow {
            operation: "reader bounds check".to_string(),
        })?;
        if end > self.data.len() {
            return Err(Error::Truncated {
                wanted: n,
                have: self.remaining(),
                offset: self.pos,
            });
        }
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        self.ensure(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_u16_le(&mut self) -> Result<u16> {
        self.ensure(2)?;
        let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        self.ensure(4)?;
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    pub fn read_u64_le(&mut self) -> Result<u64> {
        self.ensure(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        let v = u64::from_le_bytes(buf);
        self.pos += 8;
        Ok(v)
    }

    pub fn read_u16_be(&mut self) -> Result<u16> {
        self.ensure(2)?;
        let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn read_u32_be(&mut self) -> Result<u32> {
        self.ensure(4)?;
        let v = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    pub fn read_u64_be(&mut self) -> Result<u64> {
        self.ensure(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        let v = u64::from_be_bytes(buf);
        self.pos += 8;
        Ok(v)
    }

    /// Read a slice of `n` bytes without advancing past the buffer.
    pub fn read_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        self.ensure(n)?;
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Copy `n` bytes into `buf`, bounds-checked.
    pub fn read_exact_into(&mut self, buf: &mut [u8]) -> Result<()> {
        let n = buf.len();
        self.ensure(n)?;
        buf.copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(())
    }

    pub fn at_end(&self) -> bool {
        self.pos == self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_primitives_le() {
        let data: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10,
        ];
        let mut r = Reader::new(data);
        assert_eq!(r.read_u8().unwrap(), 0x01);       // byte[0]
        assert_eq!(r.read_u16_le().unwrap(), 0x0302); // bytes[1..3] little-endian
        assert_eq!(r.read_u32_le().unwrap(), 0x07060504); // bytes[3..7]
        assert_eq!(r.read_u64_le().unwrap(), 0x0f0e0d0c0b0a0908); // bytes[7..15]
        assert_eq!(r.position(), 15);
        assert!(!r.at_end());
    }

    #[test]
    fn read_primitives_be() {
        let data: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10,
        ];
        let mut r = Reader::new(data);
        assert_eq!(r.read_u16_be().unwrap(), 0x0102); // bytes[0..2]
        assert_eq!(r.read_u32_be().unwrap(), 0x03040506); // bytes[2..6]
        assert_eq!(
            r.read_u64_be().unwrap(),
            0x0708090a0b0c0d0e
        ); // bytes[6..14]
        assert_eq!(r.position(), 14);
        assert!(!r.at_end());
    }

    #[test]
    fn truncated_read_is_error() {
        let data: &[u8] = &[0x01, 0x02];
        let mut r = Reader::new(data);
        assert!(r.read_u32_le().is_err());
        assert_eq!(r.position(), 0); // cursor does not advance on failure
    }

    #[test]
    fn out_of_bounds_seek() {
        let data: &[u8] = &[0x00; 4];
        let mut r = Reader::new(data);
        assert!(r.seek(5).is_err());
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn read_slice_and_reset() {
        let data: &[u8] = &[0xaa, 0xbb, 0xcc, 0xdd];
        let mut r = Reader::new(data);
        let s = r.read_slice(2).unwrap();
        assert_eq!(s, &[0xaa, 0xbb]);
        r.reset();
        assert_eq!(r.position(), 0);
        assert_eq!(r.read_u8().unwrap(), 0xaa);
    }

    #[test]
    fn empty_reader() {
        let data: &[u8] = &[];
        let mut r = Reader::new(data);
        assert!(r.is_empty());
        assert_eq!(r.remaining(), 0);
        assert!(r.read_u8().is_err());
    }
}
