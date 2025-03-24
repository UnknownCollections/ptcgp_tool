#![allow(dead_code)]

// Constants used for shift operations in MOV instructions.
// X1 is a generic constant, while LSL_16, LSL_32, and LSL_48 represent
// the shift multipliers corresponding to 16, 32, and 48-bit shifts respectively.
pub const X1: u8 = 1;
pub const LSL_16: u8 = 1;
pub const LSL_32: u8 = 2;
pub const LSL_48: u8 = 3;

#[derive(Debug)]
pub struct Mov {
    /// Size flag (should be 1 for a 64-bit move)
    pub sf: u8,
    /// Opcode field (should be 2 for a MOVZ)
    pub opc: u8,
    /// Half‐word specifier (shift multiplier, so shift = hw * 16)
    pub hw: u8,
    /// 16-bit immediate value
    pub imm16: u16,
    /// Destination register (0-31)
    pub rd: u8,
}

#[derive(Debug)]
pub struct Movk {
    /// Size flag (should be 1 for a 64-bit move)
    pub sf: u8,
    /// Opcode field (should be 3 for a MOVK)
    pub opc: u8,
    /// Half‐word specifier (shift multiplier, so shift = hw * 16)
    pub hw: u8,
    /// 16-bit immediate value
    pub imm16: u16,
    /// Destination register (0-31)
    pub rd: u8,
}

/// Tries to parse the provided 32-bit instruction as a MOV (MOVZ) instruction.
/// For a 64-bit MOVZ we expect:
///   - sf = 1 (bit 31)
///   - opc = 10 (bits 30–29)
///   - fixed bits (28–23) = 100101 (0x25)
///     Thus, bits 31:23 should equal 0x1A5.
///     Returns Some(Mov) if the instruction matches, or None otherwise.
pub fn parse_mov(inst: u32) -> Option<Mov> {
    // Extract bits 31:23 (combining the size flag, opcode, and fixed bits).
    // These bits must match the fixed pattern for a 64-bit MOVZ instruction.
    let top9 = inst >> 23;
    if top9 != 0x1A5 {
        return None;
    }
    // Extract the size flag from bit 31.
    let sf = ((inst >> 31) & 0x1) as u8;
    // Extract the 2-bit opcode field from bits 30–29.
    let opc = ((inst >> 29) & 0x3) as u8; // should be 2 for MOVZ
    // Extract the 2-bit half-word specifier from bits 22–21.
    let hw = ((inst >> 21) & 0x3) as u8;
    // Extract the 16-bit immediate value from bits 20:5.
    let imm16 = ((inst >> 5) & 0xffff) as u16;
    // Extract the destination register from the lowest 5 bits (bits 4:0).
    let rd = (inst & 0x1F) as u8;
    Some(Mov {
        sf,
        opc,
        hw,
        imm16,
        rd,
    })
}

/// Tries to parse the provided 32-bit int as a 64-bit MOVK instruction.
/// For a 64-bit MOVK we expect:
///   - sf = 1 (bit 31)
///   - opc = 11 (bits 30–29)
///   - fixed bits (28–23) = 100101 (0x25)
///     Thus, bits 31:23 should equal 0x1E5.
///     Returns Some(Movk) if the instruction matches, or None otherwise.
pub fn parse_movk(inst: u32) -> Option<Movk> {
    // Extract bits 31:23 (combining the size flag, opcode, and fixed bits).
    // This pattern is unique to a 64-bit MOVK instruction.
    let top9 = inst >> 23;
    if top9 != 0x1E5 {
        return None;
    }
    // Extract the size flag from bit 31.
    let sf = ((inst >> 31) & 0x1) as u8;
    // Extract the 2-bit opcode field from bits 30–29.
    let opc = ((inst >> 29) & 0x3) as u8; // should be 3 for MOVK
    // Extract the 2-bit half-word specifier from bits 22–21.
    let hw = ((inst >> 21) & 0x3) as u8;
    // Extract the 16-bit immediate value from bits 20:5.
    let imm16 = ((inst >> 5) & 0xffff) as u16;
    // Extract the destination register from the lowest 5 bits (bits 4:0).
    let rd = (inst & 0x1F) as u8;
    Some(Movk {
        sf,
        opc,
        hw,
        imm16,
        rd,
    })
}
