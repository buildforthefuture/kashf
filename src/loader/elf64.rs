// Copyright (c) <OWNER>

// L1 Loader: ELF64 parser. Bounds-checked; returns Result on malformed input.
// Reference: ELF-64 Object File Format specification.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::loader::{
    Arch, BinaryFormat, LoadedBinary, Loader, Permissions, Section, Segment, Symbol,
};

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LSB: u8 = 1; // little-endian
const ELF_CURRENT_VERSION: u8 = 1;
const ELF_TYPE_EXEC: u16 = 2;
const ELF_TYPE_DYN: u16 = 3;

/// ELF section header type values.
const SHT_NULL: u32 = 0;
const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_NOBITS: u32 = 8;

const SHF_WRITE: u64 = 0x1;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;

/// ELF program header type values.
const PT_NULL: u32 = 0;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const PT_NOTE: u32 = 4;
const PT_PHDR: u32 = 6;

const PF_R: u32 = 0x4;
const PF_W: u32 = 0x2;
const PF_X: u32 = 0x1;

/// ELF symbol binding values.
const STB_LOCAL: u8 = 0;
const STB_GLOBAL: u8 = 1;
const STB_WEAK: u8 = 2;

fn st_bind(info: u8) -> u8 {
    info >> 4
}

fn st_type(info: u8) -> u8 {
    info & 0xf
}

const STT_NOTYPE: u8 = 0;
const STT_OBJECT: u8 = 1;
const STT_FUNC: u8 = 2;

pub struct Elf64Loader {
    max_sections: usize,
    max_segments: usize,
    max_symbols: usize,
}

impl Default for Elf64Loader {
    fn default() -> Self {
        Self {
            max_sections: 4096,
            max_segments: 4096,
            max_symbols: 65536,
        }
    }
}

impl Elf64Loader {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_limits(mut self, sections: usize, segments: usize, symbols: usize) -> Self {
        self.max_sections = sections;
        self.max_segments = segments;
        self.max_symbols = symbols;
        self
    }
}

impl Loader for Elf64Loader {
    fn format(&self) -> BinaryFormat {
        BinaryFormat::Elf64
    }

    fn load(&self, data: &[u8]) -> Result<LoadedBinary> {
        let mut r = Reader::new(data);

        // ---- ELF header identification ----
        let magic = r.read_slice(4)?;
        if magic != ELF_MAGIC {
            return Err(Error::BadMagic {
                offset: 0,
                expected: ELF_MAGIC,
                found: [magic[0], magic[1], magic[2], magic[3]],
            });
        }
        let class = r.read_u8()?;
        if class != ELF_CLASS_64 {
            return Err(Error::Unsupported {
                feature: "ELF class".to_string(),
                detail: format!("expected 64-bit (class 2), got class {class}"),
            });
        }
        let data_enc = r.read_u8()?;
        if data_enc != ELF_DATA_LSB {
            return Err(Error::Unsupported {
                feature: "ELF endianness".to_string(),
                detail: format!("expected little-endian (1), got {data_enc}"),
            });
        }
        let version = r.read_u8()?;
        if version != ELF_CURRENT_VERSION {
            return Err(Error::Malformed {
                what: "ELF header version".to_string(),
                offset: 7,
                detail: format!("expected {ELF_CURRENT_VERSION}, got {version}"),
            });
        }
        // OS/ABI (byte 8), ABI version (byte 9), padding (bytes 10..16)
        r.skip(8)?;

        // Bytes 16..18: e_type, 18..20: e_machine, 20..24: e_version
        let e_type = r.read_u16_le()?;
        let e_machine = r.read_u16_le()?;
        let _e_version = r.read_u32_le()?;
        // Bytes 24..32: e_entry
        let e_entry = r.read_u64_le()?;
        // Bytes 32..40: e_phoff, 40..48: e_shoff
        let e_phoff = r.read_u64_le()?;
        let e_shoff = r.read_u64_le()?;
        // Bytes 48..52: e_flags, 52..54: e_ehsize
        let _e_flags = r.read_u32_le()?;
        let e_ehsize = r.read_u16_le()?;
        // Bytes 54..56: e_phentsize, 56..58: e_phnum
        let e_phentsize = r.read_u16_le()?;
        let e_phnum = r.read_u16_le()?;
        // Bytes 58..60: e_shentsize, 60..62: e_shnum
        let e_shentsize = r.read_u16_le()?;
        let e_shnum = r.read_u16_le()?;
        // Bytes 62..64: e_shstrndx
        let e_shstrndx = r.read_u16_le()?;

        let arch = match e_machine {
            243 => Arch::Riscv64,
            62 => Arch::X86_64,
            _ => Arch::Unknown,
        };

        let mut sections = Vec::new();
        let mut segments = Vec::new();
        let mut symbols = Vec::new();

        // ---- Program headers ----
        if e_phnum as usize > self.max_segments {
            return Err(Error::ResourceLimit {
                limit: "segment count".to_string(),
                used: e_phnum as usize,
                cap: self.max_segments,
            });
        }
        if e_phnum > 0 {
            r.seek(e_phoff as usize)?;
            for i in 0..e_phnum {
                let ph_offset = e_phoff as usize + (i as usize) * (e_phentsize as usize);
                // Each program header is 56 bytes for ELF64.
                if ph_offset + 56 > data.len() {
                    return Err(Error::Truncated {
                        wanted: 56,
                        have: data.len().saturating_sub(ph_offset),
                        offset: ph_offset,
                    });
                }
                let mut pr = Reader::new(&data[ph_offset..ph_offset + 56]);
                let p_type = pr.read_u32_le()?;
                let p_flags = pr.read_u32_le()?;
                let p_offset = pr.read_u64_le()?;
                let p_vaddr = pr.read_u64_le()?;
                let _p_paddr = pr.read_u64_le()?;
                let p_filesz = pr.read_u64_le()?;
                let p_memsz = pr.read_u64_le()?;
                let _p_align = pr.read_u64_le()?;
                if p_type == PT_LOAD {
                    segments.push(Segment {
                        name: format!("segment{}", i),
                        virtual_address: p_vaddr,
                        virtual_size: p_memsz,
                        file_offset: p_offset,
                        file_size: p_filesz,
                        permissions: Permissions::from_bits(
                            ((p_flags & PF_R) >> 2 | (p_flags & PF_W) >> 1 | (p_flags & PF_X)) as u8,
                        ),
                        alignment: _p_align,
                    });
                }
            }
        }

        // ---- Section headers ----
        if e_shnum as usize > self.max_sections {
            return Err(Error::ResourceLimit {
                limit: "section count".to_string(),
                used: e_shnum as usize,
                cap: self.max_sections,
            });
        }
        let mut shstrtab_offset: u64 = 0;
        let mut shstrtab_size: u64 = 0;

        if e_shnum > 0 && e_shoff != 0 {
            r.seek(e_shoff as usize)?;
            // First pass: read all section headers into a temporary vec to
            // resolve the string table.
            let mut raw_sections: Vec<SectionHeader> = Vec::with_capacity(e_shnum as usize);
            for i in 0..e_shnum {
                let sh_offset = e_shoff as usize + (i as usize) * (e_shentsize as usize);
                if sh_offset + 64 > data.len() {
                    return Err(Error::Truncated {
                        wanted: 64,
                        have: data.len().saturating_sub(sh_offset),
                        offset: sh_offset,
                    });
                }
                let mut sr = Reader::new(&data[sh_offset..sh_offset + 64]);
                let sh_name = sr.read_u32_le()?;
                let sh_type = sr.read_u32_le()?;
                let sh_flags = sr.read_u64_le()?;
                let sh_addr = sr.read_u64_le()?;
                let sh_offset = sr.read_u64_le()?;
                let sh_size = sr.read_u64_le()?;
                let _sh_link = sr.read_u32_le()?;
                let _sh_info = sr.read_u32_le()?;
                let _sh_addralign = sr.read_u64_le()?;
                let _sh_entsize = sr.read_u64_le()?;
                raw_sections.push(SectionHeader {
                    name_idx: sh_name,
                    typ: sh_type,
                    flags: sh_flags,
                    addr: sh_addr,
                    offset: sh_offset,
                    size: sh_size,
                });
                if i == e_shstrndx as usize {
                    shstrtab_offset = sh_offset;
                    shstrtab_size = sh_size;
                }
            }

            // Second pass: materialize sections with resolved names.
            for (i, sh) in raw_sections.iter().enumerate() {
                if sections.len() >= self.max_sections {
                    return Err(Error::ResourceLimit {
                        limit: "section count".to_string(),
                        used: sections.len(),
                        cap: self.max_sections,
                    });
                }
                let name = read_string_table(
                    data,
                    shstrtab_offset,
                    shstrtab_size,
                    sh.name_idx,
                )?;
                let permissions = Permissions::from_bits(
                    ((sh.flags & SHF_WRITE) >> 1 | (sh.flags & SHF_EXECINSTR) >> 2 | (sh.flags & SHF_ALLOC) >> 1)
                        as u8,
                );
                sections.push(Section {
                    name,
                    address: sh.addr,
                    offset: sh.offset,
                    size: sh.size,
                    permissions,
                    flags: sh.flags,
                });
            }
        }

        // ---- Symbol table (first SHT_SYMTAB found) ----
        let symtab_section_idx = sections
            .iter()
            .position(|s| {
                // Re-derive type is not stored; we re-scan raw headers.
                false
            });
        let _ = symtab_section_idx;

        // Re-read section headers to find the symbol table and its strtab.
        if e_shnum > 0 && e_shoff != 0 {
            let mut sym_found = false;
            for i in 0..e_shnum {
                if sym_found {
                    break;
                }
                let sh_offset = e_shoff as usize + (i as usize) * (e_shentsize as usize);
                if sh_offset + 64 > data.len() {
                    break;
                }
                let mut sr = Reader::new(&data[sh_offset..sh_offset + 64]);
                let _sh_name = sr.read_u32_le()?;
                let sh_type = sr.read_u32_le()?;
                let _sh_flags = sr.read_u64_le()?;
                let _sh_addr = sr.read_u64_le()?;
                let _sh_offset = sr.read_u64_le()?;
                let sh_size = sr.read_u64_le()?;
                let sh_link = sr.read_u32_le()?;
                let _sh_info = sr.read_u32_le()?;
                if sh_type == SHT_SYMTAB {
                    let strtab_offset = if sh_link as usize < e_shnum as usize {
                        let str_offset =
                            e_shoff as usize + (sh_link as usize) * (e_shentsize as usize);
                        if str_offset + 64 <= data.len() {
                            let mut sr2 = Reader::new(&data[str_offset..str_offset + 64]);
                            let _ = sr2.read_u32_le();
                            let _ = sr2.read_u32_le();
                            let _ = sr2.read_u64_le();
                            let _ = sr2.read_u64_le();
                            let off = sr2.read_u64_le().unwrap_or(0);
                            off
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    parse_symtab(
                        data,
                        sh_offset,
                        sh_size,
                        strtab_offset,
                        &mut symbols,
                        self.max_symbols,
                    )?;
                    sym_found = true;
                }
            }
        }

        Ok(LoadedBinary {
            format: BinaryFormat::Elf64,
            arch,
            entry_point: e_entry,
            sections,
            segments,
            symbols,
            data: data.to_vec(),
        })
    }
}

struct SectionHeader {
    name_idx: u32,
    typ: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
}

/// Read a NUL-terminated string from the section string table.
fn read_string_table(
    data: &[u8],
    strtab_offset: u64,
    strtab_size: u64,
    idx: u32,
) -> Result<String> {
    if strtab_offset == 0 || strtab_size == 0 {
        return Ok(String::new());
    }
    let start = strtab_offset as usize + idx as usize;
    if start >= data.len() {
        return Ok(String::new());
    }
    let end = data[start..]
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(data.len() - start);
    // Cap at strtab bounds.
    let max_end = (strtab_size as usize).min(data.len() - strtab_offset as usize);
    let end = end.min(max_end.saturating_sub(idx as usize));
    let raw = &data[start..start + end];
    String::from_utf8(raw.to_vec()).map_err(|e| Error::Malformed {
        what: "section name string".to_string(),
        offset: start,
        detail: e.to_string(),
    })
}

/// Parse a symbol table section.
fn parse_symtab(
    data: &[u8],
    symtab_offset: usize,
    symtab_size: u64,
    strtab_offset: u64,
    out: &mut Vec<Symbol>,
    max_symbols: usize,
) -> Result<()> {
    const SYMENT_SIZE: usize = 24; // Elf64_Sym
    let count = (symtab_size as usize) / SYMENT_SIZE;
    if count > max_symbols {
        return Err(Error::ResourceLimit {
            limit: "symbol count".to_string(),
            used: count,
            cap: max_symbols,
        });
    }
    for i in 0..count {
        let off = symtab_offset + i * SYMENT_SIZE;
        if off + SYMENT_SIZE > data.len() {
            break;
        }
        let mut sr = Reader::new(&data[off..off + SYMENT_SIZE]);
        let st_name = sr.read_u32_le()?;
        let st_info = sr.read_u8()?;
        let _st_other = sr.read_u8()?;
        let _st_shndx = sr.read_u16_le()?;
        let st_value = sr.read_u64_le()?;
        let st_size = sr.read_u64_le()?;
        let bind = st_bind(st_info);
        let ty = st_type(st_info);
        if bind == STB_LOCAL && ty == STT_NOTYPE && st_name == 0 {
            continue; // skip null / undefined locals
        }
        let name = read_string_table(data, strtab_offset, 0, st_name)?;
        let is_function = ty == STT_FUNC;
        let is_exported = bind == STB_GLOBAL || bind == STB_WEAK;
        if out.len() >= max_symbols {
            return Err(Error::ResourceLimit {
                limit: "symbol count".to_string(),
                used: out.len(),
                cap: max_symbols,
            });
        }
        out.push(Symbol {
            name,
            address: st_value,
            size: st_size,
            section_index: 0,
            is_function,
            is_exported,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_minimal_elf64() -> Vec<u8> {
        // Construct a minimal valid ELF64 executable header with no program
        // or section headers. Enough to validate header parsing.
        let mut buf = Vec::new();
        buf.extend_from_slice(&ELF_MAGIC);
        buf.push(ELF_CLASS_64); // ei_class
        buf.push(ELF_DATA_LSB); // ei_data
        buf.push(ELF_CURRENT_VERSION); // ei_version
        buf.extend_from_slice(&[0u8; 8]); // ei_osabi..ei_pad
        buf.extend_from_slice(&(2u16).to_le_bytes()); // e_type = ET_EXEC
        buf.extend_from_slice(&(243u16).to_le_bytes()); // e_machine = EM_RISCV
        buf.extend_from_slice(&(1u32).to_le_bytes()); // e_version
        buf.extend_from_slice(&(0x1000u64).to_le_bytes()); // e_entry
        buf.extend_from_slice(&(0u64).to_le_bytes()); // e_phoff
        buf.extend_from_slice(&(0u64).to_le_bytes()); // e_shoff
        buf.extend_from_slice(&(0u32).to_le_bytes()); // e_flags
        buf.extend_from_slice(&(64u16).to_le_bytes()); // e_ehsize
        buf.extend_from_slice(&(56u16).to_le_bytes()); // e_phentsize
        buf.extend_from_slice(&(0u16).to_le_bytes()); // e_phnum
        buf.extend_from_slice(&(64u16).to_le_bytes()); // e_shentsize
        buf.extend_from_slice(&(0u16).to_le_bytes()); // e_shnum
        buf.extend_from_slice(&(0u16).to_le_bytes()); // e_shstrndx
        buf
    }

    #[test]
    fn parse_minimal_header() {
        let loader = Elf64Loader::new();
        let data = build_minimal_elf64();
        let loaded = loader.load(&data).unwrap();
        assert_eq!(loaded.format, BinaryFormat::Elf64);
        assert_eq!(loaded.arch, Arch::Riscv64);
        assert_eq!(loaded.entry_point, 0x1000);
        assert!(loaded.sections.is_empty());
        assert!(loaded.segments.is_empty());
    }

    #[test]
    fn bad_magic() {
        let loader = Elf64Loader::new();
        let err = loader.load(&[0x00, 0x00, 0x00, 0x00]).unwrap_err();
        assert!(matches!(err, Error::BadMagic { .. }));
    }

    #[test]
    fn truncated_header() {
        let loader = Elf64Loader::new();
        let data = build_minimal_elf64();
        let err = loader.load(&data[..32]).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }));
    }

    #[test]
    fn unsupported_class_32() {
        let loader = Elf64Loader::new();
        let mut data = build_minimal_elf64();
        data[4] = 1; // 32-bit class
        let err = loader.load(&data).unwrap_err();
        assert!(matches!(err, Error::Unsupported { .. }));
    }
}
