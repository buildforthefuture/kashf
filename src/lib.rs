// Copyright (c) <OWNER>

// Kashshaf: a defensive binary analyzer.
// Public crate root. Modules are small with clear public APIs and no global
// state.

pub mod error;
pub mod bytes;

pub mod loader;
pub mod decoder;
pub mod ir;
pub mod humanizer;
pub mod analyzer;
pub mod emulator;
pub mod trace;
pub mod diff;
pub mod report;
pub mod dashboard;

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// Open and read a file into memory with a configurable size cap.
pub fn read_file_capped(path: &Path, max_size: usize) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(|e| Error::Io {
        operation: "metadata".to_string(),
        source: e.to_string(),
    })?;
    let len = metadata.len() as usize;
    if len > max_size {
        return Err(Error::ResourceLimit {
            limit: "file size".to_string(),
            used: len,
            cap: max_size,
        });
    }
    let mut data = Vec::with_capacity(len);
    let mut file = fs::File::open(path).map_err(|e| Error::Io {
        operation: "open".to_string(),
        source: e.to_string(),
    })?;
    use std::io::Read;
    file.read_to_end(&mut data)
        .map_err(|e| Error::Io {
            operation: "read".to_string(),
            source: e.to_string(),
        })?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_capped_rejects_oversized() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join(format!("kashshaf_test_{}.bin", std::process::id()));
        {
            let mut f = fs::File::create(&path).unwrap();
            f.write_all(b"hello world").unwrap();
        }
        let small = read_file_capped(&path, 100).unwrap();
        assert_eq!(small, b"hello world");
        let big = read_file_capped(&path, 5);
        assert!(big.is_err());
        let _ = fs::remove_file(&path);
    }
}
