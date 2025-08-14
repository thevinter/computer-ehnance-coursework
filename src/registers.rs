use modular_bitfield::prelude::*;
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

pub enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    Sign = 0b0000_0100,
    Parity = 0b0000_1000,
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

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct RegisterRow {
    low: B8,  // Lower 8 bits
    high: B8, // Higher 8 bits
}

impl RegisterRow {
    pub fn get(self) -> u16 {
        u16::from_le_bytes(Self::into_bytes(self))
    }
}

pub struct RegisterFile {
    ax: RegisterRow, // AX (AL, AH)
    cx: RegisterRow, // CX (CL, CH)
    dx: RegisterRow, // DX (DL, DH)
    bx: RegisterRow, // BX (BL, BH)
    sp: RegisterRow, // SP
    bp: RegisterRow, // BP
    si: RegisterRow, // SI
    di: RegisterRow, // DI
    flags: u8,       // FLAGS register
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            ax: RegisterRow::new(),
            cx: RegisterRow::new(),
            dx: RegisterRow::new(),
            bx: RegisterRow::new(),
            sp: RegisterRow::new(),
            bp: RegisterRow::new(),
            si: RegisterRow::new(),
            di: RegisterRow::new(),
            flags: 0, // Initialize FLAGS to 0
        }
    }

    pub fn get(&self, reg: Register) -> u16 {
        use Register::*;
        match reg {
            AL => self.ax.low() as u16,
            CL => self.cx.low() as u16,
            DL => self.dx.low() as u16,
            BL => self.bx.low() as u16,
            AH => self.ax.high() as u16,
            CH => self.cx.high() as u16,
            DH => self.dx.high() as u16,
            BH => self.bx.high() as u16,
            AX => self.ax.get(),
            CX => self.cx.get(),
            DX => self.dx.get(),
            BX => self.bx.get(),
            SP => self.sp.get(),
            BP => self.bp.get(),
            SI => self.si.get(),
            DI => self.di.get(),
        }
    }

    pub fn set(&mut self, reg: Register, value: u16) {
        use Register::*;
        match reg {
            AL => self.ax.set_low(value as u8),
            CL => self.cx.set_low(value as u8),
            DL => self.dx.set_low(value as u8),
            BL => self.bx.set_low(value as u8),
            AH => self.ax.set_high(value as u8),
            CH => self.cx.set_high(value as u8),
            DH => self.dx.set_high(value as u8),
            BH => self.bx.set_high(value as u8),
            AX => self.ax = RegisterRow::from_bytes(value.to_le_bytes()),
            CX => self.cx = RegisterRow::from_bytes(value.to_le_bytes()),
            DX => self.dx = RegisterRow::from_bytes(value.to_le_bytes()),
            BX => self.bx = RegisterRow::from_bytes(value.to_le_bytes()),
            SP => self.sp = RegisterRow::from_bytes(value.to_le_bytes()),
            BP => self.bp = RegisterRow::from_bytes(value.to_le_bytes()),
            SI => self.si = RegisterRow::from_bytes(value.to_le_bytes()),
            DI => self.di = RegisterRow::from_bytes(value.to_le_bytes()),
        }
    }

    fn set_flag(&mut self, flag: Flag) {
        self.flags |= flag as u8;
    }

    fn clear_flag(&mut self, flag: Flag) {
        self.flags &= !(flag as u8)
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.flags & (flag as u8) != 0
    }

    pub fn print_flags(&self) {
        println!("; Flags: {:08b}", self.flags);
    }

    pub fn set_flags_from_result(&mut self, result: i16) {
        if result == 0 {
            self.set_flag(Flag::Zero);
        } else {
            self.clear_flag(Flag::Zero);
        }
        if result < 0 {
            self.set_flag(Flag::Sign);
        } else {
            self.clear_flag(Flag::Sign);
        }
        if result % 2 == 0 {
            self.set_flag(Flag::Parity);
        } else {
            self.clear_flag(Flag::Parity);
        }
    }

    pub fn raw_memory(&self) -> [u8; 16] {
        [
            self.ax.low(),
            self.ax.high(),
            self.cx.low(),
            self.cx.high(),
            self.dx.low(),
            self.dx.high(),
            self.bx.low(),
            self.bx.high(),
            self.sp.low(),
            self.sp.high(),
            self.bp.low(),
            self.bp.high(),
            self.si.low(),
            self.si.high(),
            self.di.low(),
            self.di.high(),
        ]
    }
}

pub fn retrieve_register(index: u8, w: u8) -> Result<Register, String> {
    REGISTERS
        .get(w as usize)
        .and_then(|row| row.get(index as usize))
        .copied()
        .ok_or_else(|| format!("Invalid register index: {}", index).into())
}
