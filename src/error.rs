// Copyright (c) <OWNER>

// Typed error types. Every parser/decoder/emulator operation returns a
// Result; malformed input never unwinds the stack.

use std::fmt;
use std::io;

/// A Kashshaf error. Variants carry precise context (offset, bytes, limit)
/// so callers can produce verifiable evidence rather than guesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// An underlying I/O failure while reading a file.
    Io { operation: String, source: String },
    /// The input ended before the requested number of bytes was available.
    Truncated { wanted: usize, have: usize, offset: usize },
    /// Unexpected magic bytes at a given offset.
    BadMagic { offset: usize, expected: [u8; 4], found: [u8; 4] },
    /// A requested feature or format is not supported.
    Unsupported { feature: String, detail: String },
    /// The input is structurally malformed at a given offset.
    Malformed { what: String, offset: usize, detail: String },
    /// A read would go past the end of the buffer.
    OutOfBounds { offset: usize, len: usize, max: usize },
    /// An instruction could not be decoded at an address.
    Decode { opcode: u32, address: u64, detail: String },
    /// A configurable resource limit was exceeded.
    ResourceLimit { limit: String, used: usize, cap: usize },
    /// A named item was not found.
    NotFound { what: String },
    /// An object was already initialized.
    AlreadyInitialized { what: String },
    /// Integer overflow during address/size computation.
    Overflow { operation: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { operation, source } => {
                write!(f, "I/O error during {}: {}", operation, source)
            }
            Error::Truncated { wanted, have, offset } => write!(
                f,
                "truncated input at offset 0x{offset:x}: wanted {wanted} bytes, have {have}",
            ),
            Error::BadMagic { offset, expected, found } => write!(
                f,
                "bad magic at offset {offset}: expected {:02x?}, found {:02x?}",
                expected, found,
            ),
            Error::Unsupported { feature, detail } => {
                write!(f, "unsupported {}: {}", feature, detail)
            }
            Error::Malformed { what, offset, detail } => {
                write!(f, "malformed {} at offset {offset}: {}", what, detail)
            }
            Error::OutOfBounds { offset, len, max } => {
                write!(f, "out of bounds: offset {offset} + len {len} > max {max}")
            }
            Error::Decode { opcode, address, detail } => {
                write!(
                    f,
                    "decode failure at address 0x{address:x}: opcode 0x{opcode:08x}: {detail}",
                )
            }
            Error::ResourceLimit { limit, used, cap } => {
                write!(f, "resource limit exceeded: {limit} used {used} > cap {cap}")
            }
            Error::NotFound { what } => write!(f, "not found: {what}"),
            Error::AlreadyInitialized { what } => {
                write!(f, "already initialized: {what}")
            }
            Error::Overflow { operation } => {
                write!(f, "integer overflow during {operation}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io {
            operation: "read".to_string(),
            source: err.to_string(),
        }
    }
}

/// Public result type used throughout the codebase.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_truncated() {
        let err = Error::Truncated {
            wanted: 8,
            have: 3,
            offset: 0x1000,
        };
        let s = format!("{err}");
        assert!(s.contains("0x1000"));
        assert!(s.contains("wanted 8"));
        assert!(s.contains("have 3"));
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: Error = io.into();
        match err {
            Error::Io { operation, .. } => assert_eq!(operation, "read"),
            _ => panic!("expected Io variant"),
        }
    }
}
