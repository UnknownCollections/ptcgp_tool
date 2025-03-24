#![allow(dead_code)]

// The size in bytes of a single ARM64 instruction.
pub const SIZEOF_ARM64_INSTRUCTION: usize = 4;

// The machine code bytes for the ARM64 `RET` instruction.
pub const RET_INSTRUCTION_BYTES: [u8; SIZEOF_ARM64_INSTRUCTION] = [0xC0, 0x03, 0x5F, 0xD6];


/// Represents an ARM64 general-purpose register.
///
/// # Variants
/// - **X0–X28:** General purpose registers.
/// - **X29:** Frame pointer (FP).
/// - **X30:** Link register (LR).
/// - **XZR:** Zero register (or stack pointer in some contexts)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29, // FP
    X30, // LR
    XZR, // Zero Register (or SP in some contexts)
}

impl TryFrom<u8> for Register {
    type Error = ();

    /// Attempts to convert an 8-bit unsigned integer to a [`Register`].
    ///
    /// # Returns
    /// - `Ok(Register)` if the input is within the valid range (0–31).
    /// - `Err(())` otherwise.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::X0),
            1 => Ok(Register::X1),
            2 => Ok(Register::X2),
            3 => Ok(Register::X3),
            4 => Ok(Register::X4),
            5 => Ok(Register::X5),
            6 => Ok(Register::X6),
            7 => Ok(Register::X7),
            8 => Ok(Register::X8),
            9 => Ok(Register::X9),
            10 => Ok(Register::X10),
            11 => Ok(Register::X11),
            12 => Ok(Register::X12),
            13 => Ok(Register::X13),
            14 => Ok(Register::X14),
            15 => Ok(Register::X15),
            16 => Ok(Register::X16),
            17 => Ok(Register::X17),
            18 => Ok(Register::X18),
            19 => Ok(Register::X19),
            20 => Ok(Register::X20),
            21 => Ok(Register::X21),
            22 => Ok(Register::X22),
            23 => Ok(Register::X23),
            24 => Ok(Register::X24),
            25 => Ok(Register::X25),
            26 => Ok(Register::X26),
            27 => Ok(Register::X27),
            28 => Ok(Register::X28),
            29 => Ok(Register::X29),
            30 => Ok(Register::X30),
            31 => Ok(Register::XZR),
            _ => Err(()),
        }
    }
}

/// Represents the possible left shift amounts used for encoding immediate values in ARM64 instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftAmount {
    Lsl0,
    Lsl16,
    Lsl32,
    Lsl48,
}

impl ShiftAmount {
    /// Returns the 2-bit encoding (0–3) corresponding to the half-word field.
    pub fn to_u8(&self) -> u8 {
        match self {
            ShiftAmount::Lsl0 => 0,
            ShiftAmount::Lsl16 => 1,
            ShiftAmount::Lsl32 => 2,
            ShiftAmount::Lsl48 => 3,
        }
    }

    /// Computes the actual shift amount in bits by multiplying the encoded value by 16.
    pub fn to_shift_bits(&self) -> u8 {
        self.to_u8() * 16
    }
}

impl TryFrom<u8> for ShiftAmount {
    type Error = ();

    /// Attempts to convert an 8-bit unsigned integer to a [`ShiftAmount`].
    ///
    /// # Returns
    /// - `Ok(ShiftAmount)` if the input is within 0–3.
    /// - `Err(())` otherwise.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShiftAmount::Lsl0),
            1 => Ok(ShiftAmount::Lsl16),
            2 => Ok(ShiftAmount::Lsl32),
            3 => Ok(ShiftAmount::Lsl48),
            _ => Err(()),
        }
    }
}

/// Represents the MOV (register) variant of a MOV instruction in ARM64.
///
/// This variant is an alias for the ORR instruction and follows a specific encoding.
#[derive(Debug)]
pub struct MovRegister {
    /// Size flag (bit 31) indicating the data size.
    pub sf: u8,
    /// Source register containing the value to be moved.
    pub rm: Register,
    /// Destination register to which the value is moved.
    pub rd: Register,
}

/// Represents the MOV (bitmask immediate) variant of a MOV instruction in ARM64.
///
/// This instruction writes a bitmask immediate value into a register and follows a specific encoding.
#[derive(Debug)]
pub struct MovBitmaskImmediate {
    /// Size flag (bit 31) indicating the data size.
    pub sf: u8,
    /// Bitmask immediate field N.
    pub n: u8,
    /// Bitmask immediate field `immr`.
    pub immr: u8,
    /// Bitmask immediate field `imms`.
    pub imms: u8,
    /// Destination register.
    pub rd: Register,
}

/// Represents a MOV instruction in ARM64.
///
/// This enum differentiates between the two variants of the MOV instruction:
/// - Register variant: uses a register as the source.
/// - Bitmask immediate variant: uses a bitmask immediate value.
#[derive(Debug)]
pub enum Mov {
    /// MOV instruction using a register (alias for ORR).
    Register(MovRegister),
    /// MOV instruction using a bitmask immediate value.
    BitmaskImmediate(MovBitmaskImmediate),
}

impl Mov {
    /// Returns the destination register of the MOV instruction.
    pub fn rd(&self) -> Register {
        match self {
            Mov::Register(r) => r.rd,
            Mov::BitmaskImmediate(bi) => bi.rd,
        }
    }
}

/// Parses a 32-bit instruction word into a [`Mov`] instruction variant.
///
/// The function distinguishes between the register and bitmask immediate variants
/// by examining fixed bit patterns and validating reserved fields.
///
/// # Parameters
/// - `inst`: A 32-bit unsigned integer representing the encoded instruction.
///
/// # Returns
/// - `Some(Mov)` if the instruction matches one of the MOV encodings.
/// - `None` if the encoding does not match either variant.
pub fn parse_mov(inst: u32) -> Option<Mov> {
    // For both variants, bits 9–5 (Rn) must equal 11111 (ZR/WZR).
    if ((inst >> 5) & 0x1F) != 0x1F {
        return None;
    }
    // Extract the size flag.
    let sf = ((inst >> 31) & 0x1) as u8;

    // Check for the register variant.
    // This variant uses a 7-bit fixed pattern in bits 30–24 equal to 0x2A (binary 0101010).
    if ((inst >> 24) & 0x7F) == 0x2A {
        // Validate that the shift field (bits 23–22) is 00.
        if ((inst >> 22) & 0x3) != 0 {
            return None;
        }
        // Validate that bit 21 (N) is 0.
        if ((inst >> 21) & 0x1) != 0 {
            return None;
        }
        // Validate that the imm6 field (bits 15–10) is 000000.
        if ((inst >> 10) & 0x3F) != 0 {
            return None;
        }
        // Extract source register (rm) from bits 20–16.
        let rm_val = ((inst >> 16) & 0x1F) as u8;
        // Extract destination register (rd) from bits 4–0.
        let rd_val = (inst & 0x1F) as u8;
        let rm = Register::try_from(rm_val).ok()?;
        let rd = Register::try_from(rd_val).ok()?;
        return Some(Mov::Register(MovRegister { sf, rm, rd }));
    }

    // Check for the bitmask immediate variant.
    // Here, bits 30–23 (8 bits) must equal 0x64 (binary 01100100).
    if ((inst >> 23) & 0xFF) == 0x64 {
        // Bit 22 is the N field.
        let n = ((inst >> 22) & 0x1) as u8;
        // Bits 21–16 are the immr field.
        let immr = ((inst >> 16) & 0x3F) as u8;
        // Bits 15–10 are the imms field.
        let imms = ((inst >> 10) & 0x3F) as u8;
        // Bits 4–0 are the destination register (rd).
        let rd_val = (inst & 0x1F) as u8;
        let rd = Register::try_from(rd_val).ok()?;
        return Some(Mov::BitmaskImmediate(MovBitmaskImmediate {
            sf,
            n,
            immr,
            imms,
            rd,
        }));
    }

    // If neither pattern matches, the instruction does not represent a valid MOV.
    None
}

impl MovBitmaskImmediate {
    /// Decodes and returns the immediate value for a MOV (bitmask immediate) instruction.
    ///
    /// The algorithm follows the A64 “DecodeBitMasks” specification:
    /// 1. Determine the `len` field from the concatenation of `n` and `imms`.
    /// 2. Compute the element size (`esize`); if `len == 6` for a 64-bit immediate, `esize` is forced to 64,
    ///    otherwise `esize = 1 << (len + 1)`.
    /// 3. Determine the number of consecutive ones (d + 1) from the lower bits of `imms`.
    /// 4. Create the basic bitmask, rotate it right by `immr` (modulo `esize`), and replicate it
    ///    to fill the register width (32 or 64 bits).
    pub fn imm(&self) -> u64 {
        // Determine the register size based on the size flag.
        let reg_size = if self.sf == 1 { 64 } else { 32 };

        // Combine N and imms to determine "len". The 7-bit value has bit6 = N and bits5:0 = imms.
        let combined: u32 = ((self.n as u32) << 6) | (self.imms as u32);
        let mut len = 0;
        // Find the position of the most-significant set bit in the combined value.
        for i in (0..7).rev() {
            if ((combined >> i) & 1) == 1 {
                len = i;
                break;
            }
        }

        // Compute the element size. For len == 6 in a 64-bit immediate, esize is forced to 64.
        let esize = if len == 6 { 64 } else { 1 << (len + 1) };

        // Calculate a mask to extract the lower (len+1) bits of imms.
        let mask = (1 << (len + 1)) - 1;
        let d = ((self.imms as u32) & mask) + 1; // d = number of ones

        // Create a bitmask with d ones.
        let mut welem: u64 = if d >= 64 { u64::MAX } else { (1u64 << d) - 1 };

        // Rotate the bitmask right by immr (with rotation amount modulo esize).
        let r = (self.immr as u32) % esize;
        welem = welem.rotate_right(r);

        // Replicate the element to fill the entire register.
        let repeats = reg_size / esize;
        let mut imm: u64 = 0;
        for i in 0..repeats {
            imm |= welem << (i * esize);
        }

        imm
    }
}

/// Represents a 64-bit MOVZ (move wide with zero) instruction in ARM64.
///
/// This instruction moves a 16-bit immediate value into a register while zeroing the remaining bits.
#[derive(Debug)]
pub struct Movz {
    /// Size flag; should be 1 for a 64-bit move.
    pub sf: u8,
    /// Opcode field; should be 0xA5 for a MOVZ instruction.
    pub opc: u8,
    /// Half‐word specifier as a shift amount.
    pub hw: ShiftAmount,
    /// 16-bit immediate value.
    pub imm16: u16,
    /// Destination register.
    pub rd: Register,
}

/// Attempts to parse a 32-bit instruction as a 64-bit MOVZ instruction.
///
/// # Parameters
/// - `inst`: A 32-bit unsigned integer representing the encoded instruction.
///
/// # Returns
/// - `Some(MovZ)` if the instruction matches the MOVZ encoding.
/// - `None` otherwise.
pub fn parse_movz(inst: u32) -> Option<Movz> {
    if ((inst >> 23) & 0xFF) != 0xA5 {
        return None;
    }
    let sf = ((inst >> 31) & 0x1) as u8;
    let opc = ((inst >> 29) & 0x3) as u8;
    let hw_val = ((inst >> 21) & 0x3) as u8;
    let hw = ShiftAmount::try_from(hw_val).ok()?;
    let imm16 = ((inst >> 5) & 0xffff) as u16;
    let rd_val = (inst & 0x1F) as u8;
    let rd = Register::try_from(rd_val).ok()?;
    Some(Movz {
        sf,
        opc,
        hw,
        imm16,
        rd,
    })
}

/// Represents a 64-bit MOVK (move wide with keep) instruction in ARM64.
///
/// This instruction writes a 16-bit immediate value into a register while preserving
/// other bits from the original register value.
#[derive(Debug)]
pub struct Movk {
    /// Size flag; should be 1 for a 64-bit move.
    pub sf: u8,
    /// Opcode field; should be 0xE5 for a MOVK instruction.
    pub opc: u8,
    /// Half‐word specifier as a shift amount.
    pub hw: ShiftAmount,
    /// 16-bit immediate value.
    pub imm16: u16,
    /// Destination register.
    pub rd: Register,
}

/// Attempts to parse a 32-bit instruction as a 64-bit MOVK instruction.
///
/// # Parameters
/// - `inst`: A 32-bit unsigned integer representing the encoded instruction.
///
/// # Returns
/// - `Some(Movk)` if the instruction matches the MOVK encoding.
/// - `None` otherwise.
pub fn parse_movk(inst: u32) -> Option<Movk> {
    if ((inst >> 23) & 0xFF) != 0xE5 {
        return None;
    }
    let sf = ((inst >> 31) & 0x1) as u8;
    let opc = ((inst >> 29) & 0x3) as u8;
    let hw_val = ((inst >> 21) & 0x3) as u8;
    let hw = ShiftAmount::try_from(hw_val).ok()?;
    let imm16 = ((inst >> 5) & 0xffff) as u16;
    let rd_val = (inst & 0x1F) as u8;
    let rd = Register::try_from(rd_val).ok()?;
    Some(Movk {
        sf,
        opc,
        hw,
        imm16,
        rd,
    })
}

/// Represents a MADD (multiply–add) instruction in ARM64.
///
/// This instruction multiplies two registers and then adds a third register value.
/// Its encoding contains:
/// - A size flag to select 32- or 64-bit operation.
/// - A fixed pattern in bits 30–21.
/// - Three source registers and one destination register.
#[derive(Debug)]
pub struct Madd {
    /// Size flag (bit 31): determines the data size, where data size = 32 << sf.
    pub sf: u8,
    /// Second source register (multiplier).
    pub rm: Register,
    /// Third source register (addend).
    pub ra: Register,
    /// First source register (multiplicand).
    pub rn: Register,
    /// Destination register.
    pub rd: Register,
}

/// Attempts to parse a 32-bit instruction as a MADD (multiply–add) instruction.
///
/// # Parameters
/// - `inst`: A 32-bit unsigned integer representing the encoded instruction.
///
/// # Returns
/// - `Some(Madd)` if the instruction matches the MADD encoding.
/// - `None` otherwise.
pub fn parse_madd(inst: u32) -> Option<Madd> {
    // Verify that bits 30–21 match the fixed pattern 0b0011011000.
    if ((inst >> 21) & 0x3FF) != 0b0011011000 {
        return None;
    }
    // Ensure that bit 15 is 0 as required.
    if ((inst >> 15) & 0x1) != 0 {
        return None;
    }
    let sf = ((inst >> 31) & 0x1) as u8;
    let rm_val = ((inst >> 16) & 0x1F) as u8;
    let ra_val = ((inst >> 10) & 0x1F) as u8;
    let rn_val = ((inst >> 5) & 0x1F) as u8;
    let rd_val = (inst & 0x1F) as u8;
    let rm = Register::try_from(rm_val).ok()?;
    let ra = Register::try_from(ra_val).ok()?;
    let rn = Register::try_from(rn_val).ok()?;
    let rd = Register::try_from(rd_val).ok()?;
    Some(Madd { sf, rm, ra, rn, rd })
}

/// Represents a 64-bit MOVN (move wide with NOT) instruction in ARM64.
///
/// This instruction moves the bitwise inverse of an optionally shifted 16-bit immediate value
/// into a register.
#[derive(Debug)]
pub struct Movn {
    /// Size flag; should be 1 for a 64-bit move.
    pub sf: u8,
    /// Opcode field; for MOVN, bits 30-23 should be 0x25.
    pub opc: u8,
    /// Half‐word specifier as a shift amount.
    pub hw: ShiftAmount,
    /// 16-bit immediate value.
    pub imm16: u16,
    /// Destination register.
    pub rd: Register,
}

/// Attempts to parse a 32-bit instruction as a MOVN instruction.
///
/// # Parameters
/// - `inst`: A 32-bit unsigned integer representing the encoded instruction.
///
/// # Returns
/// - `Some(MovN)` if the instruction matches the MOVN encoding.
/// - `None` otherwise.
pub fn parse_movn(inst: u32) -> Option<Movn> {
    // Extract the size flag (bit 31)
    let sf = ((inst >> 31) & 0x1) as u8;
    // For MOVN, the top 9 bits (sf and opcode) must match the expected value.
    let top9 = inst >> 23;
    let expected_top9 = ((sf as u32) << 8) | 0x25;
    if top9 != expected_top9 {
        return None;
    }
    // opc field (bits 30-23) – should be 0x25.
    let opc = ((inst >> 23) & 0xFF) as u8;
    // Extract the half‐word field (bits 22-21).
    let hw_val = ((inst >> 21) & 0x3) as u8;
    // For the 32-bit variant (sf == 0), only shifts 0 and 16 are allowed.
    if sf == 0 && (hw_val >> 1) == 1 {
        return None;
    }
    let hw = ShiftAmount::try_from(hw_val).ok()?;
    // Extract the 16-bit immediate (bits 20..5).
    let imm16 = ((inst >> 5) & 0xffff) as u16;
    // Extract the destination register (bits 4..0).
    let rd_val = (inst & 0x1F) as u8;
    let rd = Register::try_from(rd_val).ok()?;
    Some(Movn {
        sf,
        opc,
        hw,
        imm16,
        rd,
    })
}
