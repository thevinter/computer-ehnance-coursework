use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    MovRmR, // Register or Memory to Register
    MovIR,  // Immediate to Register
    MovIRm, // Immediate to Register or Memory
    MovAM,  // Accumulator to Memory
    MovMA,  // Memory to Accumulator
    AddRmR,
    AddIRm,
    AddIA,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::MovRmR | Opcode::MovIR | Opcode::MovIRm | Opcode::MovAM | Opcode::MovMA => {
                write!(f, "mov")
            }
            Opcode::AddRmR | Opcode::AddIRm | Opcode::AddIA => write!(f, "add"),
        }
    }
}

#[derive(Default)]
pub struct BitTrie {
    value: Option<Opcode>,
    children: HashMap<u8, BitTrie>,
}

impl BitTrie {
    pub fn insert(&mut self, bits: u8, len: u8, val: Opcode) {
        let mut node = self;
        for i in (0..len).rev() {
            let bit = (bits >> i) & 1;
            node = node.children.entry(bit).or_default();
        }
        node.value = Some(val);
    }

    pub fn match_bits(&self, byte: u8) -> Option<(Opcode, u8)> {
        let mut node = self;
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            if let Some(next) = node.children.get(&bit) {
                node = next;
                if let Some(val) = node.value {
                    let matched_len = 8 - i;
                    return Some((val, matched_len));
                }
            } else {
                break;
            }
        }
        None
    }
}

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

pub struct Reader {
    buffer: Vec<u8>,
    pos: usize,
}

impl Reader {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(Self { buffer, pos: 0 })
    }

    pub fn peek(&self) -> Option<u8> {
        self.buffer.get(self.pos).copied()
    }

    pub fn read_n(&mut self, n: usize) -> Option<&[u8]> {
        if self.pos + n <= self.buffer.len() {
            let chunk = &self.buffer[self.pos..self.pos + n];
            self.pos += n;
            Some(chunk)
        } else {
            None
        }
    }
}

pub trait IteratorExt: Iterator<Item = u8> {
    fn next_or_exit(&mut self, context: &str) -> u8 {
        self.next().unwrap_or_else(|| {
            eprintln!("Unexpected end of file while reading {}", context);
            std::process::exit(1);
        })
    }
}

impl<I: Iterator<Item = u8>> IteratorExt for I {}

impl Iterator for Reader {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.buffer.len() {
            let byte = self.buffer[self.pos];
            self.pos += 1;
            Some(byte)
        } else {
            None
        }
    }
}
