use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Register {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Register::AL => "al",
            Register::CL => "cl",
            Register::DL => "dl",
            Register::BL => "bl",
            Register::AH => "ah",
            Register::CH => "ch",
            Register::DH => "dh",
            Register::BH => "bh",
            Register::AX => "ax",
            Register::CX => "cx",
            Register::DX => "dx",
            Register::BX => "bx",
            Register::SP => "sp",
            Register::BP => "bp",
            Register::SI => "si",
            Register::DI => "di",
        };
        write!(f, "{}", s)
    }
}

pub static REGISTERS: [[Register; 8]; 2] = [
    [
        Register::AL, // 0b000
        Register::CL, // 0b001
        Register::DL, // 0b010
        Register::BL, // 0b011
        Register::AH, // 0b100
        Register::CH, // 0b101
        Register::DH, // 0b110
        Register::BH, // 0b111
    ],
    [
        Register::AX,
        Register::CX,
        Register::DX,
        Register::BX,
        Register::SP,
        Register::BP,
        Register::SI,
        Register::DI,
    ],
];

#[derive(Copy, Clone)]
pub enum EAC {
    BXSI,
    BXDI,
    BPSI,
    BPDI,
    SI,
    DI,
    BPOrDA,
    BX,
}

impl fmt::Display for EAC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EAC::BXSI => "bx + si",
            EAC::BXDI => "bx + di",
            EAC::BPSI => "bp + si",
            EAC::BPDI => "bp + di",
            EAC::SI => "si",
            EAC::DI => "di",
            EAC::BPOrDA => "bp",
            EAC::BX => "bx",
        };
        write!(f, "{}", s)
    }
}

pub static EACS: [EAC; 8] = [
    EAC::BXSI,   // 0b00
    EAC::BXDI,   // 0b01
    EAC::BPSI,   // 0b10
    EAC::BPDI,   // 0b11
    EAC::SI,     // 0b100
    EAC::DI,     // 0b101
    EAC::BPOrDA, // 0b110
    EAC::BX,     // 0b111
];

pub fn retrieve_register(index: u8, w: u8) -> Result<Register, String> {
    REGISTERS
        .get(w as usize)
        .and_then(|row| row.get(index as usize))
        .copied()
        .ok_or_else(|| format!("Invalid register index: {}", index).into())
}
