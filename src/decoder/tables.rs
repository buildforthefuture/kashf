// Copyright (c) <OWNER>

// L2 Decoder: data tables for RV64I. Pure data, separated from decode logic.
// Register/CSR names follow the RISC-V ABI convention.

/// ABI names for integer registers x0..x31.
pub const REG_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1",
    "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

/// Common CSR register names (address -> name).
pub const CSR_NAMES: &[(u16, &str)] = &[
    (0x001, "fflags"),
    (0x002, "frm"),
    (0x003, "fcsr"),
    (0xc00, "cycle"),
    (0xc01, "time"),
    (0xc02, "instret"),
    (0xc80, "cycleh"),
    (0xc81, "timeh"),
    (0xc82, "instreth"),
];

pub fn reg_name(r: u8) -> &'static str {
    REG_NAMES[r as usize]
}

pub fn csr_name(csr: u16) -> &'static str {
    CSR_NAMES
        .iter()
        .find(|(a, _)| *a == csr)
        .map(|(_, n)| *n)
        .unwrap_or("csr")
}

/// Sign-extend a value of `bits` width to i64.
pub fn sext(val: u64, bits: u32) -> i64 {
    if bits == 0 || bits >= 64 {
        return val as i64;
    }
    let sign = 1u64 << (bits - 1);
    let mask = (1u64 << bits) - 1;
    let v = val & mask;
    if v & sign != 0 {
        (v | !mask) as i64
    } else {
        v as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_names_count() {
        assert_eq!(REG_NAMES.len(), 32);
        assert_eq!(reg_name(0), "zero");
        assert_eq!(reg_name(2), "sp");
        assert_eq!(reg_name(31), "t6");
    }

    #[test]
    fn sext_positive() {
        assert_eq!(sext(0x000, 12), 0);
        assert_eq!(sext(0x7ff, 12), 2047);
    }

    #[test]
    fn sext_negative() {
        // 12-bit 0x800 = -2048
        assert_eq!(sext(0x800, 12), -2048);
        // 12-bit 0xfff = -1
        assert_eq!(sext(0xfff, 12), -1);
    }

    #[test]
    fn csr_name_known_unknown() {
        assert_eq!(csr_name(0xc01), "time");
        assert_eq!(csr_name(0x1234), "csr");
    }
}
