//! CPU Instructions
//!
//! This module defines instruction types, addressing modes, and the instruction table.

use crate::common::Byte;

/// CPU instruction types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    None,
    Nop,
    Ld,
    Inc,
    Dec,
    Rlca,
    Add,
    Rrca,
    Stop,
    Rla,
    Jr,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    Halt,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
    Pop,
    Jp,
    Push,
    Ret,
    Cb,
    Call,
    Reti,
    Ldh,
    Di,
    Ei,
    Rst,
    // CB-prefixed instructions
    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
    Bit,
    Res,
    Set,
}


/// Addressing modes for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Register,
    RegisterRegister,
    MemoryRegister,
    RegisterMemory,
    RegisterD8,
    RegisterD16,
    RegisterA8,
    RegisterA16,
    A8Register,
    A16Register,
    MemoryRegisterD8,
    HliRegister,
    HldRegister,
    RegisterHli,
    RegisterHld,
    HlSpr,
    D8,
    D16,
    MemoryRegisterOnly,
}

/// Register types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterType {
    None,
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    Af,
    Bc,
    De,
    Hl,
    Sp,
    Pc,
}

/// Condition types for conditional instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionType {
    None,
    Nz,
    Z,
    Nc,
    C,
}


/// CPU instruction definition
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub inst_type: InstructionType,
    pub mode: AddressingMode,
    pub reg1: RegisterType,
    pub reg2: RegisterType,
    pub cond: ConditionType,
    pub param: Byte,
}

impl Instruction {
    pub const fn new() -> Self {
        Self {
            inst_type: InstructionType::None,
            mode: AddressingMode::Implied,
            reg1: RegisterType::None,
            reg2: RegisterType::None,
            cond: ConditionType::None,
            param: 0,
        }
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Self::new()
    }
}

// Helper macro for instruction definition
macro_rules! inst {
    ($t:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::Implied, reg1: RegisterType::None, reg2: RegisterType::None, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, $m:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::$m, reg1: RegisterType::None, reg2: RegisterType::None, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, $m:ident, $r1:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::$m, reg1: RegisterType::$r1, reg2: RegisterType::None, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, $m:ident, $r1:ident, $r2:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::$m, reg1: RegisterType::$r1, reg2: RegisterType::$r2, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, $m:ident, $r1:ident, $r2:ident, $c:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::$m, reg1: RegisterType::$r1, reg2: RegisterType::$r2, cond: ConditionType::$c, param: 0 }
    };
    ($t:ident, $m:ident, $r1:ident, $r2:ident, $c:ident, $p:expr) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::$m, reg1: RegisterType::$r1, reg2: RegisterType::$r2, cond: ConditionType::$c, param: $p }
    };
}


/// Main instruction table (256 entries)
pub static INSTRUCTIONS: [Instruction; 256] = [
    // 0x00 - 0x0F
    inst!(Nop),                                          // 0x00
    inst!(Ld, RegisterD16, Bc),                          // 0x01
    inst!(Ld, MemoryRegister, Bc, A),                    // 0x02
    inst!(Inc, Register, Bc),                            // 0x03
    inst!(Inc, Register, B),                             // 0x04
    inst!(Dec, Register, B),                             // 0x05
    inst!(Ld, RegisterD8, B),                            // 0x06
    inst!(Rlca),                                         // 0x07
    inst!(Ld, A16Register, None, Sp),                    // 0x08
    inst!(Add, RegisterRegister, Hl, Bc),                // 0x09
    inst!(Ld, RegisterMemory, A, Bc),                    // 0x0A
    inst!(Dec, Register, Bc),                            // 0x0B
    inst!(Inc, Register, C),                             // 0x0C
    inst!(Dec, Register, C),                             // 0x0D
    inst!(Ld, RegisterD8, C),                            // 0x0E
    inst!(Rrca),                                         // 0x0F
    // 0x10 - 0x1F
    inst!(Stop),                                         // 0x10
    inst!(Ld, RegisterD16, De),                          // 0x11
    inst!(Ld, MemoryRegister, De, A),                    // 0x12
    inst!(Inc, Register, De),                            // 0x13
    inst!(Inc, Register, D),                             // 0x14
    inst!(Dec, Register, D),                             // 0x15
    inst!(Ld, RegisterD8, D),                            // 0x16
    inst!(Rla),                                          // 0x17
    inst!(Jr, D8),                                       // 0x18
    inst!(Add, RegisterRegister, Hl, De),                // 0x19
    inst!(Ld, RegisterMemory, A, De),                    // 0x1A
    inst!(Dec, Register, De),                            // 0x1B
    inst!(Inc, Register, E),                             // 0x1C
    inst!(Dec, Register, E),                             // 0x1D
    inst!(Ld, RegisterD8, E),                            // 0x1E
    inst!(Rra),                                          // 0x1F
    // 0x20 - 0x2F
    inst!(Jr, D8, None, None, Nz),                       // 0x20
    inst!(Ld, RegisterD16, Hl),                          // 0x21
    inst!(Ld, HliRegister, Hl, A),                       // 0x22
    inst!(Inc, Register, Hl),                            // 0x23
    inst!(Inc, Register, H),                             // 0x24
    inst!(Dec, Register, H),                             // 0x25
    inst!(Ld, RegisterD8, H),                            // 0x26
    inst!(Daa),                                          // 0x27
    inst!(Jr, D8, None, None, Z),                        // 0x28
    inst!(Add, RegisterRegister, Hl, Hl),                // 0x29
    inst!(Ld, RegisterHli, A, Hl),                       // 0x2A
    inst!(Dec, Register, Hl),                            // 0x2B
    inst!(Inc, Register, L),                             // 0x2C
    inst!(Dec, Register, L),                             // 0x2D
    inst!(Ld, RegisterD8, L),                            // 0x2E
    inst!(Cpl),                                          // 0x2F
    // 0x30 - 0x3F
    inst!(Jr, D8, None, None, Nc),                       // 0x30
    inst!(Ld, RegisterD16, Sp),                          // 0x31
    inst!(Ld, HldRegister, Hl, A),                       // 0x32
    inst!(Inc, Register, Sp),                            // 0x33
    inst!(Inc, MemoryRegisterOnly, Hl),                  // 0x34
    inst!(Dec, MemoryRegisterOnly, Hl),                  // 0x35
    inst!(Ld, MemoryRegisterD8, Hl),                     // 0x36
    inst!(Scf),                                          // 0x37
    inst!(Jr, D8, None, None, C),                        // 0x38
    inst!(Add, RegisterRegister, Hl, Sp),                // 0x39
    inst!(Ld, RegisterHld, A, Hl),                       // 0x3A
    inst!(Dec, Register, Sp),                            // 0x3B
    inst!(Inc, Register, A),                             // 0x3C
    inst!(Dec, Register, A),                             // 0x3D
    inst!(Ld, RegisterD8, A),                            // 0x3E
    inst!(Ccf),                                          // 0x3F
    // 0x40 - 0x4F (LD B,r and LD C,r)
    inst!(Ld, RegisterRegister, B, B),                   // 0x40
    inst!(Ld, RegisterRegister, B, C),                   // 0x41
    inst!(Ld, RegisterRegister, B, D),                   // 0x42
    inst!(Ld, RegisterRegister, B, E),                   // 0x43
    inst!(Ld, RegisterRegister, B, H),                   // 0x44
    inst!(Ld, RegisterRegister, B, L),                   // 0x45
    inst!(Ld, RegisterMemory, B, Hl),                    // 0x46
    inst!(Ld, RegisterRegister, B, A),                   // 0x47
    inst!(Ld, RegisterRegister, C, B),                   // 0x48
    inst!(Ld, RegisterRegister, C, C),                   // 0x49
    inst!(Ld, RegisterRegister, C, D),                   // 0x4A
    inst!(Ld, RegisterRegister, C, E),                   // 0x4B
    inst!(Ld, RegisterRegister, C, H),                   // 0x4C
    inst!(Ld, RegisterRegister, C, L),                   // 0x4D
    inst!(Ld, RegisterMemory, C, Hl),                    // 0x4E
    inst!(Ld, RegisterRegister, C, A),                   // 0x4F
    // 0x50 - 0x5F (LD D,r and LD E,r)
    inst!(Ld, RegisterRegister, D, B),                   // 0x50
    inst!(Ld, RegisterRegister, D, C),                   // 0x51
    inst!(Ld, RegisterRegister, D, D),                   // 0x52
    inst!(Ld, RegisterRegister, D, E),                   // 0x53
    inst!(Ld, RegisterRegister, D, H),                   // 0x54
    inst!(Ld, RegisterRegister, D, L),                   // 0x55
    inst!(Ld, RegisterMemory, D, Hl),                    // 0x56
    inst!(Ld, RegisterRegister, D, A),                   // 0x57
    inst!(Ld, RegisterRegister, E, B),                   // 0x58
    inst!(Ld, RegisterRegister, E, C),                   // 0x59
    inst!(Ld, RegisterRegister, E, D),                   // 0x5A
    inst!(Ld, RegisterRegister, E, E),                   // 0x5B
    inst!(Ld, RegisterRegister, E, H),                   // 0x5C
    inst!(Ld, RegisterRegister, E, L),                   // 0x5D
    inst!(Ld, RegisterMemory, E, Hl),                    // 0x5E
    inst!(Ld, RegisterRegister, E, A),                   // 0x5F
    // 0x60 - 0x6F (LD H,r and LD L,r)
    inst!(Ld, RegisterRegister, H, B),                   // 0x60
    inst!(Ld, RegisterRegister, H, C),                   // 0x61
    inst!(Ld, RegisterRegister, H, D),                   // 0x62
    inst!(Ld, RegisterRegister, H, E),                   // 0x63
    inst!(Ld, RegisterRegister, H, H),                   // 0x64
    inst!(Ld, RegisterRegister, H, L),                   // 0x65
    inst!(Ld, RegisterMemory, H, Hl),                    // 0x66
    inst!(Ld, RegisterRegister, H, A),                   // 0x67
    inst!(Ld, RegisterRegister, L, B),                   // 0x68
    inst!(Ld, RegisterRegister, L, C),                   // 0x69
    inst!(Ld, RegisterRegister, L, D),                   // 0x6A
    inst!(Ld, RegisterRegister, L, E),                   // 0x6B
    inst!(Ld, RegisterRegister, L, H),                   // 0x6C
    inst!(Ld, RegisterRegister, L, L),                   // 0x6D
    inst!(Ld, RegisterMemory, L, Hl),                    // 0x6E
    inst!(Ld, RegisterRegister, L, A),                   // 0x6F
    // 0x70 - 0x7F (LD (HL),r and LD A,r)
    inst!(Ld, MemoryRegister, Hl, B),                    // 0x70
    inst!(Ld, MemoryRegister, Hl, C),                    // 0x71
    inst!(Ld, MemoryRegister, Hl, D),                    // 0x72
    inst!(Ld, MemoryRegister, Hl, E),                    // 0x73
    inst!(Ld, MemoryRegister, Hl, H),                    // 0x74
    inst!(Ld, MemoryRegister, Hl, L),                    // 0x75
    inst!(Halt),                                         // 0x76
    inst!(Ld, MemoryRegister, Hl, A),                    // 0x77
    inst!(Ld, RegisterRegister, A, B),                   // 0x78
    inst!(Ld, RegisterRegister, A, C),                   // 0x79
    inst!(Ld, RegisterRegister, A, D),                   // 0x7A
    inst!(Ld, RegisterRegister, A, E),                   // 0x7B
    inst!(Ld, RegisterRegister, A, H),                   // 0x7C
    inst!(Ld, RegisterRegister, A, L),                   // 0x7D
    inst!(Ld, RegisterMemory, A, Hl),                    // 0x7E
    inst!(Ld, RegisterRegister, A, A),                   // 0x7F
    // 0x80 - 0x8F (ADD A,r and ADC A,r)
    inst!(Add, RegisterRegister, A, B),                  // 0x80
    inst!(Add, RegisterRegister, A, C),                  // 0x81
    inst!(Add, RegisterRegister, A, D),                  // 0x82
    inst!(Add, RegisterRegister, A, E),                  // 0x83
    inst!(Add, RegisterRegister, A, H),                  // 0x84
    inst!(Add, RegisterRegister, A, L),                  // 0x85
    inst!(Add, RegisterMemory, A, Hl),                   // 0x86
    inst!(Add, RegisterRegister, A, A),                  // 0x87
    inst!(Adc, RegisterRegister, A, B),                  // 0x88
    inst!(Adc, RegisterRegister, A, C),                  // 0x89
    inst!(Adc, RegisterRegister, A, D),                  // 0x8A
    inst!(Adc, RegisterRegister, A, E),                  // 0x8B
    inst!(Adc, RegisterRegister, A, H),                  // 0x8C
    inst!(Adc, RegisterRegister, A, L),                  // 0x8D
    inst!(Adc, RegisterMemory, A, Hl),                   // 0x8E
    inst!(Adc, RegisterRegister, A, A),                  // 0x8F
    // 0x90 - 0x9F (SUB A,r and SBC A,r)
    inst!(Sub, RegisterRegister, A, B),                  // 0x90
    inst!(Sub, RegisterRegister, A, C),                  // 0x91
    inst!(Sub, RegisterRegister, A, D),                  // 0x92
    inst!(Sub, RegisterRegister, A, E),                  // 0x93
    inst!(Sub, RegisterRegister, A, H),                  // 0x94
    inst!(Sub, RegisterRegister, A, L),                  // 0x95
    inst!(Sub, RegisterMemory, A, Hl),                   // 0x96
    inst!(Sub, RegisterRegister, A, A),                  // 0x97
    inst!(Sbc, RegisterRegister, A, B),                  // 0x98
    inst!(Sbc, RegisterRegister, A, C),                  // 0x99
    inst!(Sbc, RegisterRegister, A, D),                  // 0x9A
    inst!(Sbc, RegisterRegister, A, E),                  // 0x9B
    inst!(Sbc, RegisterRegister, A, H),                  // 0x9C
    inst!(Sbc, RegisterRegister, A, L),                  // 0x9D
    inst!(Sbc, RegisterMemory, A, Hl),                   // 0x9E
    inst!(Sbc, RegisterRegister, A, A),                  // 0x9F
    // 0xA0 - 0xAF (AND A,r and XOR A,r)
    inst!(And, RegisterRegister, A, B),                  // 0xA0
    inst!(And, RegisterRegister, A, C),                  // 0xA1
    inst!(And, RegisterRegister, A, D),                  // 0xA2
    inst!(And, RegisterRegister, A, E),                  // 0xA3
    inst!(And, RegisterRegister, A, H),                  // 0xA4
    inst!(And, RegisterRegister, A, L),                  // 0xA5
    inst!(And, RegisterMemory, A, Hl),                   // 0xA6
    inst!(And, RegisterRegister, A, A),                  // 0xA7
    inst!(Xor, RegisterRegister, A, B),                  // 0xA8
    inst!(Xor, RegisterRegister, A, C),                  // 0xA9
    inst!(Xor, RegisterRegister, A, D),                  // 0xAA
    inst!(Xor, RegisterRegister, A, E),                  // 0xAB
    inst!(Xor, RegisterRegister, A, H),                  // 0xAC
    inst!(Xor, RegisterRegister, A, L),                  // 0xAD
    inst!(Xor, RegisterMemory, A, Hl),                   // 0xAE
    inst!(Xor, RegisterRegister, A, A),                  // 0xAF
    // 0xB0 - 0xBF (OR A,r and CP A,r)
    inst!(Or, RegisterRegister, A, B),                   // 0xB0
    inst!(Or, RegisterRegister, A, C),                   // 0xB1
    inst!(Or, RegisterRegister, A, D),                   // 0xB2
    inst!(Or, RegisterRegister, A, E),                   // 0xB3
    inst!(Or, RegisterRegister, A, H),                   // 0xB4
    inst!(Or, RegisterRegister, A, L),                   // 0xB5
    inst!(Or, RegisterMemory, A, Hl),                    // 0xB6
    inst!(Or, RegisterRegister, A, A),                   // 0xB7
    inst!(Cp, RegisterRegister, A, B),                   // 0xB8
    inst!(Cp, RegisterRegister, A, C),                   // 0xB9
    inst!(Cp, RegisterRegister, A, D),                   // 0xBA
    inst!(Cp, RegisterRegister, A, E),                   // 0xBB
    inst!(Cp, RegisterRegister, A, H),                   // 0xBC
    inst!(Cp, RegisterRegister, A, L),                   // 0xBD
    inst!(Cp, RegisterMemory, A, Hl),                    // 0xBE
    inst!(Cp, RegisterRegister, A, A),                   // 0xBF
    // 0xC0 - 0xCF
    inst!(Ret, Implied, None, None, Nz),                 // 0xC0
    inst!(Pop, Register, Bc),                            // 0xC1
    inst!(Jp, D16, None, None, Nz),                      // 0xC2
    inst!(Jp, D16),                                      // 0xC3
    inst!(Call, D16, None, None, Nz),                    // 0xC4
    inst!(Push, Register, Bc),                           // 0xC5
    inst!(Add, RegisterD8, A),                           // 0xC6
    inst!(Rst, Implied, None, None, None, 0x00),         // 0xC7
    inst!(Ret, Implied, None, None, Z),                  // 0xC8
    inst!(Ret),                                          // 0xC9
    inst!(Jp, D16, None, None, Z),                       // 0xCA
    inst!(Cb, D8),                                       // 0xCB
    inst!(Call, D16, None, None, Z),                     // 0xCC
    inst!(Call, D16),                                    // 0xCD
    inst!(Adc, RegisterD8, A),                           // 0xCE
    inst!(Rst, Implied, None, None, None, 0x08),         // 0xCF
    // 0xD0 - 0xDF
    inst!(Ret, Implied, None, None, Nc),                 // 0xD0
    inst!(Pop, Register, De),                            // 0xD1
    inst!(Jp, D16, None, None, Nc),                      // 0xD2
    inst!(None),                                         // 0xD3 (invalid)
    inst!(Call, D16, None, None, Nc),                    // 0xD4
    inst!(Push, Register, De),                           // 0xD5
    inst!(Sub, RegisterD8, A),                           // 0xD6
    inst!(Rst, Implied, None, None, None, 0x10),         // 0xD7
    inst!(Ret, Implied, None, None, C),                  // 0xD8
    inst!(Reti),                                         // 0xD9
    inst!(Jp, D16, None, None, C),                       // 0xDA
    inst!(None),                                         // 0xDB (invalid)
    inst!(Call, D16, None, None, C),                     // 0xDC
    inst!(None),                                         // 0xDD (invalid)
    inst!(Sbc, RegisterD8, A),                           // 0xDE
    inst!(Rst, Implied, None, None, None, 0x18),         // 0xDF
    // 0xE0 - 0xEF
    inst!(Ldh, A8Register, None, A),                     // 0xE0
    inst!(Pop, Register, Hl),                            // 0xE1
    inst!(Ld, MemoryRegister, C, A),                     // 0xE2
    inst!(None),                                         // 0xE3 (invalid)
    inst!(None),                                         // 0xE4 (invalid)
    inst!(Push, Register, Hl),                           // 0xE5
    inst!(And, RegisterD8, A),                           // 0xE6
    inst!(Rst, Implied, None, None, None, 0x20),         // 0xE7
    inst!(Add, RegisterD8, Sp),                          // 0xE8
    inst!(Jp, Register, Hl),                             // 0xE9
    inst!(Ld, A16Register, None, A),                     // 0xEA
    inst!(None),                                         // 0xEB (invalid)
    inst!(None),                                         // 0xEC (invalid)
    inst!(None),                                         // 0xED (invalid)
    inst!(Xor, RegisterD8, A),                           // 0xEE
    inst!(Rst, Implied, None, None, None, 0x28),         // 0xEF
    // 0xF0 - 0xFF
    inst!(Ldh, RegisterA8, A),                           // 0xF0
    inst!(Pop, Register, Af),                            // 0xF1
    inst!(Ld, RegisterMemory, A, C),                     // 0xF2
    inst!(Di),                                           // 0xF3
    inst!(None),                                         // 0xF4 (invalid)
    inst!(Push, Register, Af),                           // 0xF5
    inst!(Or, RegisterD8, A),                            // 0xF6
    inst!(Rst, Implied, None, None, None, 0x30),         // 0xF7
    inst!(Ld, HlSpr, Hl, Sp),                            // 0xF8
    inst!(Ld, RegisterRegister, Sp, Hl),                 // 0xF9
    inst!(Ld, RegisterA16, A),                           // 0xFA
    inst!(Ei),                                           // 0xFB
    inst!(None),                                         // 0xFC (invalid)
    inst!(None),                                         // 0xFD (invalid)
    inst!(Cp, RegisterD8, A),                            // 0xFE
    inst!(Rst, Implied, None, None, None, 0x38),         // 0xFF
];

/// Get instruction by opcode
pub fn instruction_by_opcode(opcode: Byte) -> &'static Instruction {
    &INSTRUCTIONS[opcode as usize]
}


// Helper macro for CB instructions
macro_rules! cb_inst {
    ($t:ident, $r:ident) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::Register, reg1: RegisterType::$r, reg2: RegisterType::None, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, $r:ident, $bit:expr) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::Register, reg1: RegisterType::$r, reg2: RegisterType::None, cond: ConditionType::None, param: $bit }
    };
    ($t:ident, Hl) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::MemoryRegisterOnly, reg1: RegisterType::Hl, reg2: RegisterType::None, cond: ConditionType::None, param: 0 }
    };
    ($t:ident, Hl, $bit:expr) => {
        Instruction { inst_type: InstructionType::$t, mode: AddressingMode::MemoryRegisterOnly, reg1: RegisterType::Hl, reg2: RegisterType::None, cond: ConditionType::None, param: $bit }
    };
}

/// CB-prefixed instruction table (256 entries)
pub static CB_INSTRUCTIONS: [Instruction; 256] = [
    // 0x00 - 0x07: RLC r
    cb_inst!(Rlc, B), cb_inst!(Rlc, C), cb_inst!(Rlc, D), cb_inst!(Rlc, E),
    cb_inst!(Rlc, H), cb_inst!(Rlc, L), cb_inst!(Rlc, Hl), cb_inst!(Rlc, A),
    // 0x08 - 0x0F: RRC r
    cb_inst!(Rrc, B), cb_inst!(Rrc, C), cb_inst!(Rrc, D), cb_inst!(Rrc, E),
    cb_inst!(Rrc, H), cb_inst!(Rrc, L), cb_inst!(Rrc, Hl), cb_inst!(Rrc, A),
    // 0x10 - 0x17: RL r
    cb_inst!(Rl, B), cb_inst!(Rl, C), cb_inst!(Rl, D), cb_inst!(Rl, E),
    cb_inst!(Rl, H), cb_inst!(Rl, L), cb_inst!(Rl, Hl), cb_inst!(Rl, A),
    // 0x18 - 0x1F: RR r
    cb_inst!(Rr, B), cb_inst!(Rr, C), cb_inst!(Rr, D), cb_inst!(Rr, E),
    cb_inst!(Rr, H), cb_inst!(Rr, L), cb_inst!(Rr, Hl), cb_inst!(Rr, A),
    // 0x20 - 0x27: SLA r
    cb_inst!(Sla, B), cb_inst!(Sla, C), cb_inst!(Sla, D), cb_inst!(Sla, E),
    cb_inst!(Sla, H), cb_inst!(Sla, L), cb_inst!(Sla, Hl), cb_inst!(Sla, A),
    // 0x28 - 0x2F: SRA r
    cb_inst!(Sra, B), cb_inst!(Sra, C), cb_inst!(Sra, D), cb_inst!(Sra, E),
    cb_inst!(Sra, H), cb_inst!(Sra, L), cb_inst!(Sra, Hl), cb_inst!(Sra, A),
    // 0x30 - 0x37: SWAP r
    cb_inst!(Swap, B), cb_inst!(Swap, C), cb_inst!(Swap, D), cb_inst!(Swap, E),
    cb_inst!(Swap, H), cb_inst!(Swap, L), cb_inst!(Swap, Hl), cb_inst!(Swap, A),
    // 0x38 - 0x3F: SRL r
    cb_inst!(Srl, B), cb_inst!(Srl, C), cb_inst!(Srl, D), cb_inst!(Srl, E),
    cb_inst!(Srl, H), cb_inst!(Srl, L), cb_inst!(Srl, Hl), cb_inst!(Srl, A),
    // 0x40 - 0x47: BIT 0,r
    cb_inst!(Bit, B, 0), cb_inst!(Bit, C, 0), cb_inst!(Bit, D, 0), cb_inst!(Bit, E, 0),
    cb_inst!(Bit, H, 0), cb_inst!(Bit, L, 0), cb_inst!(Bit, Hl, 0), cb_inst!(Bit, A, 0),
    // 0x48 - 0x4F: BIT 1,r
    cb_inst!(Bit, B, 1), cb_inst!(Bit, C, 1), cb_inst!(Bit, D, 1), cb_inst!(Bit, E, 1),
    cb_inst!(Bit, H, 1), cb_inst!(Bit, L, 1), cb_inst!(Bit, Hl, 1), cb_inst!(Bit, A, 1),
    // 0x50 - 0x57: BIT 2,r
    cb_inst!(Bit, B, 2), cb_inst!(Bit, C, 2), cb_inst!(Bit, D, 2), cb_inst!(Bit, E, 2),
    cb_inst!(Bit, H, 2), cb_inst!(Bit, L, 2), cb_inst!(Bit, Hl, 2), cb_inst!(Bit, A, 2),
    // 0x58 - 0x5F: BIT 3,r
    cb_inst!(Bit, B, 3), cb_inst!(Bit, C, 3), cb_inst!(Bit, D, 3), cb_inst!(Bit, E, 3),
    cb_inst!(Bit, H, 3), cb_inst!(Bit, L, 3), cb_inst!(Bit, Hl, 3), cb_inst!(Bit, A, 3),
    // 0x60 - 0x67: BIT 4,r
    cb_inst!(Bit, B, 4), cb_inst!(Bit, C, 4), cb_inst!(Bit, D, 4), cb_inst!(Bit, E, 4),
    cb_inst!(Bit, H, 4), cb_inst!(Bit, L, 4), cb_inst!(Bit, Hl, 4), cb_inst!(Bit, A, 4),
    // 0x68 - 0x6F: BIT 5,r
    cb_inst!(Bit, B, 5), cb_inst!(Bit, C, 5), cb_inst!(Bit, D, 5), cb_inst!(Bit, E, 5),
    cb_inst!(Bit, H, 5), cb_inst!(Bit, L, 5), cb_inst!(Bit, Hl, 5), cb_inst!(Bit, A, 5),
    // 0x70 - 0x77: BIT 6,r
    cb_inst!(Bit, B, 6), cb_inst!(Bit, C, 6), cb_inst!(Bit, D, 6), cb_inst!(Bit, E, 6),
    cb_inst!(Bit, H, 6), cb_inst!(Bit, L, 6), cb_inst!(Bit, Hl, 6), cb_inst!(Bit, A, 6),
    // 0x78 - 0x7F: BIT 7,r
    cb_inst!(Bit, B, 7), cb_inst!(Bit, C, 7), cb_inst!(Bit, D, 7), cb_inst!(Bit, E, 7),
    cb_inst!(Bit, H, 7), cb_inst!(Bit, L, 7), cb_inst!(Bit, Hl, 7), cb_inst!(Bit, A, 7),
    // 0x80 - 0x87: RES 0,r
    cb_inst!(Res, B, 0), cb_inst!(Res, C, 0), cb_inst!(Res, D, 0), cb_inst!(Res, E, 0),
    cb_inst!(Res, H, 0), cb_inst!(Res, L, 0), cb_inst!(Res, Hl, 0), cb_inst!(Res, A, 0),
    // 0x88 - 0x8F: RES 1,r
    cb_inst!(Res, B, 1), cb_inst!(Res, C, 1), cb_inst!(Res, D, 1), cb_inst!(Res, E, 1),
    cb_inst!(Res, H, 1), cb_inst!(Res, L, 1), cb_inst!(Res, Hl, 1), cb_inst!(Res, A, 1),
    // 0x90 - 0x97: RES 2,r
    cb_inst!(Res, B, 2), cb_inst!(Res, C, 2), cb_inst!(Res, D, 2), cb_inst!(Res, E, 2),
    cb_inst!(Res, H, 2), cb_inst!(Res, L, 2), cb_inst!(Res, Hl, 2), cb_inst!(Res, A, 2),
    // 0x98 - 0x9F: RES 3,r
    cb_inst!(Res, B, 3), cb_inst!(Res, C, 3), cb_inst!(Res, D, 3), cb_inst!(Res, E, 3),
    cb_inst!(Res, H, 3), cb_inst!(Res, L, 3), cb_inst!(Res, Hl, 3), cb_inst!(Res, A, 3),
    // 0xA0 - 0xA7: RES 4,r
    cb_inst!(Res, B, 4), cb_inst!(Res, C, 4), cb_inst!(Res, D, 4), cb_inst!(Res, E, 4),
    cb_inst!(Res, H, 4), cb_inst!(Res, L, 4), cb_inst!(Res, Hl, 4), cb_inst!(Res, A, 4),
    // 0xA8 - 0xAF: RES 5,r
    cb_inst!(Res, B, 5), cb_inst!(Res, C, 5), cb_inst!(Res, D, 5), cb_inst!(Res, E, 5),
    cb_inst!(Res, H, 5), cb_inst!(Res, L, 5), cb_inst!(Res, Hl, 5), cb_inst!(Res, A, 5),
    // 0xB0 - 0xB7: RES 6,r
    cb_inst!(Res, B, 6), cb_inst!(Res, C, 6), cb_inst!(Res, D, 6), cb_inst!(Res, E, 6),
    cb_inst!(Res, H, 6), cb_inst!(Res, L, 6), cb_inst!(Res, Hl, 6), cb_inst!(Res, A, 6),
    // 0xB8 - 0xBF: RES 7,r
    cb_inst!(Res, B, 7), cb_inst!(Res, C, 7), cb_inst!(Res, D, 7), cb_inst!(Res, E, 7),
    cb_inst!(Res, H, 7), cb_inst!(Res, L, 7), cb_inst!(Res, Hl, 7), cb_inst!(Res, A, 7),
    // 0xC0 - 0xC7: SET 0,r
    cb_inst!(Set, B, 0), cb_inst!(Set, C, 0), cb_inst!(Set, D, 0), cb_inst!(Set, E, 0),
    cb_inst!(Set, H, 0), cb_inst!(Set, L, 0), cb_inst!(Set, Hl, 0), cb_inst!(Set, A, 0),
    // 0xC8 - 0xCF: SET 1,r
    cb_inst!(Set, B, 1), cb_inst!(Set, C, 1), cb_inst!(Set, D, 1), cb_inst!(Set, E, 1),
    cb_inst!(Set, H, 1), cb_inst!(Set, L, 1), cb_inst!(Set, Hl, 1), cb_inst!(Set, A, 1),
    // 0xD0 - 0xD7: SET 2,r
    cb_inst!(Set, B, 2), cb_inst!(Set, C, 2), cb_inst!(Set, D, 2), cb_inst!(Set, E, 2),
    cb_inst!(Set, H, 2), cb_inst!(Set, L, 2), cb_inst!(Set, Hl, 2), cb_inst!(Set, A, 2),
    // 0xD8 - 0xDF: SET 3,r
    cb_inst!(Set, B, 3), cb_inst!(Set, C, 3), cb_inst!(Set, D, 3), cb_inst!(Set, E, 3),
    cb_inst!(Set, H, 3), cb_inst!(Set, L, 3), cb_inst!(Set, Hl, 3), cb_inst!(Set, A, 3),
    // 0xE0 - 0xE7: SET 4,r
    cb_inst!(Set, B, 4), cb_inst!(Set, C, 4), cb_inst!(Set, D, 4), cb_inst!(Set, E, 4),
    cb_inst!(Set, H, 4), cb_inst!(Set, L, 4), cb_inst!(Set, Hl, 4), cb_inst!(Set, A, 4),
    // 0xE8 - 0xEF: SET 5,r
    cb_inst!(Set, B, 5), cb_inst!(Set, C, 5), cb_inst!(Set, D, 5), cb_inst!(Set, E, 5),
    cb_inst!(Set, H, 5), cb_inst!(Set, L, 5), cb_inst!(Set, Hl, 5), cb_inst!(Set, A, 5),
    // 0xF0 - 0xF7: SET 6,r
    cb_inst!(Set, B, 6), cb_inst!(Set, C, 6), cb_inst!(Set, D, 6), cb_inst!(Set, E, 6),
    cb_inst!(Set, H, 6), cb_inst!(Set, L, 6), cb_inst!(Set, Hl, 6), cb_inst!(Set, A, 6),
    // 0xF8 - 0xFF: SET 7,r
    cb_inst!(Set, B, 7), cb_inst!(Set, C, 7), cb_inst!(Set, D, 7), cb_inst!(Set, E, 7),
    cb_inst!(Set, H, 7), cb_inst!(Set, L, 7), cb_inst!(Set, Hl, 7), cb_inst!(Set, A, 7),
];

/// Get CB-prefixed instruction by opcode
pub fn cb_instruction_by_opcode(opcode: Byte) -> &'static Instruction {
    &CB_INSTRUCTIONS[opcode as usize]
}
