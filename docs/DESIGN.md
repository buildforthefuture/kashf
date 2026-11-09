# Kashshaf Design Reference Document

This document lists the official specification sections used as authoritative
references for each layer of Kashshaf. All implementations are clean-room and
original; no third-party source code was copied, quoted, or paraphrased.

## Reference conventions
- `Intel SDM` = Intel 64 and IA-32 Architectures Software Developer's Manual.
- `Arm ARM` = Arm Architecture Reference Manual (Armv8-A / Armv9-A).
- `RISC-V Spec` = The RISC-V Instruction Set Manual, Volume I (Unprivileged
  ISA) and Volume II (Privileged ISA).
- `ELF` = ELF-64 Object File Format specification.
- `PE/COFF` = Microsoft PE/COFF specification.
- `Mach-O` = Apple Mach-O file format reference.

---

## L1 Loader

### ELF64
Reference: ELF specification.
- `e_ident[0..15]` magic/class/endianness/version/osabi.
- `e_type`, `e_machine` (EM_RISCV=243, EM_X86_64=62).
- `e_entry`, `e_phoff`, `e_shoff`, `e_ehsize`, `e_phentsize`, `e_phnum`,
  `e_shentsize`, `e_shnum`, `e_shstrndx`.
- Program header (56 bytes): p_type, p_flags, p_offset, p_vaddr, p_paddr,
  p_filesz, p_memsz, p_align. PT_LOAD segments.
- Section header (64 bytes): sh_name, sh_type, sh_flags, sh_addr, sh_offset,
  sh_size, sh_link, sh_info, sh_addralign, sh_entsize.
- Symbol table (Elf64_Sym, 24 bytes): st_name, st_info, st_other, st_shndx,
  st_value, st_size. SHT_SYMTAB/SHT_STRTAB/SHT_NOBITS.
- Permissions from PF_R/PF_W/PF_X and SHF_WRITE/SHF_ALLOC/SHF_EXECINSTR.

### PE / Mach-O / Raw
Placeholders for Phases 7-8. Reference: PE/COFF spec, Mach-O spec.

### Resource limits
Max section count, segment count, symbol count, file size — all configurable,
bounds-checked before allocation.

---

## L2 Decoder (RV64I first)

Reference: RISC-V ISA manual, Volume I, Chapter 2 (RV64I base integer ISA).

### Instruction formats
- R-type: opcode[6:0], rd[11:7], funct3[14:12], rs1[19:15], rs2[24:20],
  funct7[31:25].
- I-type: opcode, rd, funct3, rs1, imm[31:20] (sign-extended 12-bit).
- S-type: opcode, imm[11:5]=[31:25], rs2, rs1, funct3, imm[4:0]=[11:7].
- B-type: opcode, imm[12]=[31], imm[11]=[7], imm[10:5]=[30:25],
  imm[4:1]=[11:8], imm[0]=0; rs2, rs1, funct3.
- U-type: opcode, rd, imm[31:12].
- J-type: opcode, rd, imm[20]=[31], imm[19:12]=[19:12], imm[11]=[20],
  imm[10:1]=[30:21], imm[0]=0.

### Immediates
All sign-extended per width (I/S: 12-bit, B: 13-bit, U: 20-bit shifted,
J: 21-bit). Implemented via `sext(val, bits)`.

### Registers
x0=zero ... x31 per RISC-V ABI naming table (original table).

### CSRs
Original table of common CSR addresses (fflags=0x001, frm=0x002, fcsr=0x003,
cycle=0xc00, time=0xc01, instret=0xc02, ...).

### Decoder guarantees
- Fixed 4-byte instructions.
- Unknown opcode/funct => `Error::Decode` with address + opcode.
- Truncated input => `Error::Truncated`.
- No `unsafe`, no panics on malformed input.

---

## L3 IR

Original design. An effect-oriented intermediate representation.

- `IrNode { address: u64, effect: IrEffect }` — one effectful operation at a
  source address.
- `IrEffect` variants: RegRead, RegWrite, MemLoad, MemStore, Branch, Jump,
  Call, Return, Syscall, Nop.
- `IrExpr` variants: Const, Reg, Mem, Add, Sub, And, Or, Xor, Shl, Shr, Sra,
  Eq, Ne, Lt, Ltu, Neg. Every variant records its source address.
- `IrFunction { name, address, nodes: Vec<IrNode> }` — a lifted function.
- Lifting: RV64I instructions map to one or more IR nodes (e.g. `jal` writes
  the return-address register then jumps). Branch/jump targets are computed
  as absolute virtual addresses.

Reference for semantics: RISC-V ISA manual (instruction behavior). The IR
structure itself is original.

## L4 Humanizer

Original design. IR to plain-text / pseudo-code.

- `Humanizer` trait: `language()`, `format_node()`, `format_function()`.
- `Language { En, Ar }`.
- Every emitted sentence references the source IR node address (traceability).
- English template implemented first; Arabic template is a stub for Phase 2
  (ar) follow-up.

---

## L5 Analyzer
Placeholder (Phase 3-5). CFG, basic blocks, stack-frame recovery, data-flow,
taint tracking, pattern detectors.

---

## L6 Emulator
Placeholder (Phase 3). Sandboxed software emulation over virtual memory map
with permission checks. Never executes native code. Emits compact binary
trace events.

---

## L7 Trace Store
Placeholder (Phase 3). Append-only event log with index for seek/filter.

---

## L8 Diff
Placeholder (Phase 6). Function matching via structural CFG hash plus
normalized instruction sequences.

---

## L9 Report
Placeholder (Phase 5). Findings JSON + plain text with severity, address,
evidence, explanation, fix suggestion, confidence level.

---

## L10 Dashboard
Placeholder (Phase 4-6). Single self-contained HTML file, inline CSS/JS,
SVG/Canvas rendering, no libraries. Optional local std::net server on
127.0.0.1 only.

---

## Cross-cutting constraints
- Zero dependencies: Rust standard library only (`cargo build` offline with
  empty dependency list).
- No networking, telemetry, or auto-update. Air-gapped.
- Deterministic: same input => byte-identical output.
- Untrusted input: every parser bounds-checked, typed `Result`, no panics.
- No `unsafe` without documented justification.
- Native code is never executed; tracing runs in a sandboxed software
  emulator.
- Clean-room original code; neutral header "Copyright (c) <OWNER>"; no
  third-party license headers, credits, or repository links.
