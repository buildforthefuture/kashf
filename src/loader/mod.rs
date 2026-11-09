// Copyright (c) <OWNER>

// L1 Loader: shared structures and traits for ELF/PE/Mach-O/raw loaders.
// All loaders are bounds-checked and return Result; no panics on malformed
// input.

use crate::error::{Error, Result};

/// Permissions on a section or segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }
    pub fn from_bits(bits: u8) -> Self {
        Self {
            read: bits & 0x4 != 0,
            write: bits & 0x2 != 0,
            execute: bits & 0x1 != 0,
        }
    }
    pub fn to_bits(self) -> u8 {
        (if self.read { 4 } else { 0 })
            | (if self.write { 2 } else { 0 })
            | (if self.execute { 1 } else { 0 })
    }
    pub fn is_writable_and_executable(self) -> bool {
        self.write && self.execute
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        s.push(if self.read { 'r' } else { '-' });
        s.push(if self.write { 'w' } else { '-' });
        s.push(if self.execute { 'x' } else { '-' });
        write!(f, "{s}")
    }
}

use std::fmt;

/// A named section of a binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    /// Virtual address where the section is loaded.
    pub address: u64,
    /// File offset of the section contents.
    pub offset: u64,
    pub size: u64,
    pub permissions: Permissions,
    pub flags: u64,
}

/// A loadable segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub file_offset: u64,
    pub file_size: u64,
    pub permissions: Permissions,
    pub alignment: u64,
}

/// A symbol table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub section_index: usize,
    pub is_function: bool,
    pub is_exported: bool,
}

/// The binary format of an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Elf64,
    Pe,
    MachO,
    Raw,
}

/// The target architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Arch {
    #[default]
    Unknown,
    Riscv64,
    X86_64,
    Arm64,
}

/// The fully loaded binary, independent of source format.
#[derive(Debug, Clone)]
pub struct LoadedBinary {
    pub format: BinaryFormat,
    pub arch: Arch,
    pub entry_point: u64,
    pub sections: Vec<Section>,
    pub segments: Vec<Segment>,
    pub symbols: Vec<Symbol>,
    /// Raw file bytes (kept for re-parsing / hashing).
    pub data: Vec<u8>,
}

impl LoadedBinary {
    pub fn section_by_name(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }
    pub fn symbol_by_address(&self, address: u64) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.address == address)
    }
    pub fn symbols_in_range(&self, start: u64, end: u64) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.address >= start && s.address < end)
            .collect()
    }
}

/// Common loader trait.
pub trait Loader {
    fn format(&self) -> BinaryFormat;
    fn load(&self, data: &[u8]) -> Result<LoadedBinary>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_bits() {
        let p = Permissions::from_bits(0b101);
        assert!(p.read);
        assert!(!p.write);
        assert!(p.execute);
        assert_eq!(p.to_bits(), 0b101);
        assert!(p.is_writable_and_executable() == false);
        let wx = Permissions::from_bits(0b011);
        assert!(wx.is_writable_and_executable());
    }
}
