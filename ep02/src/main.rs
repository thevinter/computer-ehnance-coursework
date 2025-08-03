use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::fs::{self};
use std::io::{self, Read};
use std::path::Path;

#[derive(Copy, Clone, Debug)]
enum Opcode {
    MovRmR,
    MovIR,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::MovRmR | Opcode::MovIR => write!(f, "mov"),
        }
    }
}

#[derive(Default)]
struct BitTrie {
    value: Option<Opcode>,
    children: HashMap<u8, BitTrie>,
}

impl BitTrie {
    fn insert(&mut self, bits: u8, len: u8, val: Opcode) {
        let mut node = self;
        for i in (0..len).rev() {
            let bit = (bits >> i) & 1;
            node = node.children.entry(bit).or_default();
        }
        node.value = Some(val);
    }

    fn match_bits(&self, byte: u8) -> Option<(Opcode, u8)> {
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

static OPCODE_TRIE: Lazy<BitTrie> = Lazy::new(|| {
    let mut trie = BitTrie::default();
    trie.insert(0b100010, 6, Opcode::MovRmR);
    trie.insert(0b1011, 4, Opcode::MovIR);
    trie
});

#[derive(Copy, Clone, Debug)]
enum Register {
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

static REGISTERS: [[Register; 8]; 2] = [
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
enum EAC {
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

static EACS: [EAC; 8] = [
    EAC::BXSI,   // 0b00
    EAC::BXDI,   // 0b01
    EAC::BPSI,   // 0b10
    EAC::BPDI,   // 0b11
    EAC::SI,     // 0b100
    EAC::DI,     // 0b101
    EAC::BPOrDA, // 0b110
    EAC::BX,     // 0b111
];

struct Reader {
    buffer: Vec<u8>,
    pos: usize,
}

impl Reader {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(Self { buffer, pos: 0 })
    }

    fn peek(&self) -> Option<u8> {
        self.buffer.get(self.pos).copied()
    }

    fn read_n(&mut self, n: usize) -> Option<&[u8]> {
        if self.pos + n <= self.buffer.len() {
            let chunk = &self.buffer[self.pos..self.pos + n];
            self.pos += n;
            Some(chunk)
        } else {
            None
        }
    }
}

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

fn main() {
    let mut args = env::args();
    args.next();

    let path = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!(
                "Usage: {} <path_to_file>",
                args.next().unwrap_or_else(|| "program".to_string())
            );
            std::process::exit(1);
        }
    };

    let file = match read_file(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let mut reader = Reader::new(&path).unwrap_or_else(|e| {
        eprintln!("Failed to create reader: {}", e);
        std::process::exit(1);
    });

    println!("; File read successfully, size: {} bytes", file.len());

    while let Some(chunk) = reader.next() {
        let b0 = chunk;

        let opcode = OPCODE_TRIE
            .match_bits(b0)
            .map(|(opcode, _)| opcode)
            .unwrap_or_else(|| {
                eprintln!("Unknown opcode: {}", b0);
                std::process::exit(1);
            });

        match opcode {
            Opcode::MovRmR => {
                let b1 = reader.next().unwrap_or_else(|| {
                    eprintln!("Unexpected end of file while reading opcode");
                    std::process::exit(1);
                });
                let mode = b1 >> 6;
                if mode == 0b01 {
                    let b2 = reader.next().unwrap_or_else(|| {
                        eprintln!("Unexpected end of file while reading MOV RMR");
                        std::process::exit(1);
                    });
                    //println!("Mode with {:08b},{:08b}, {:08b}, ", b0, b1, b2);
                    process_mov_rmr(&[b0, b1, b2], 3);
                } else if mode == 0b10 {
                    let b2 = reader.next().unwrap_or_else(|| {
                        eprintln!("Unexpected end of file while reading MOV RMR");
                        std::process::exit(1);
                    });
                    let b3 = reader.next().unwrap_or_else(|| {
                        eprintln!("Unexpected end of file while reading MOV RMR");
                        std::process::exit(1);
                    });
                    //println!("Mode with {:08b},{:08b}, {:08b}, {:08b}", b0, b1, b2, b3);
                    process_mov_rmr(&[b0, b1, b2, b3], 4);
                } else {
                    //println!("Mode with {:08b},{:08b},", b0, b1,);
                    process_mov_rmr(&[b0, b1], 2);
                }
            }
            Opcode::MovIR => {
                let w = (b0 >> 3) & 0b1;
                let b1 = reader.next().unwrap_or_else(|| {
                    eprintln!("Unexpected end of file while reading opcode");
                    std::process::exit(1);
                });
                if w == 1 {
                    let b2 = reader.next().unwrap_or_else(|| {
                        eprintln!("Unexpected end of file while reading MOV IR");
                        std::process::exit(1);
                    });
                    process_move_ir(&[b0, b1, b2], 3);
                } else {
                    process_move_ir(&[b0, b1], 2);
                }
            }
        };
    }
}

fn read_file(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

fn retrieve_register(index: u8, w: u8) -> Result<Register, String> {
    REGISTERS
        .get(w as usize)
        .and_then(|row| row.get(index as usize))
        .copied()
        .ok_or_else(|| format!("Invalid register index: {}", index).into())
}

fn process_move_ir(bytes: &[u8], size: u8) {
    let w = (bytes[0] >> 3) & 0b1;
    let reg = bytes[0] & 0b111;
    let value = if size == 3 {
        ((bytes[2] as i16) << 8) | (bytes[1] as i16)
    } else {
        bytes[1] as i16
    };

    let dest = retrieve_register(reg, w).expect("Failed to get destination register");

    println!("mov {}, {}", dest, value);
}

fn process_mov_rmr(bytes: &[u8], size: u8) {
    assert!(size >= 2);
    let d = (bytes[0] >> 1) & 0b1;
    let w = bytes[0] & 0b1;

    let mode = bytes[1] >> 6;
    let reg = (bytes[1] >> 3) & 0b111;
    let regormem = bytes[1] & 0b111;

    if mode == 0b00 {
        let source = EACS[regormem as usize];
        let dest = retrieve_register(reg, w).expect("Failed to get source register");
        if d == 1 {
            println!("mov {}, [{}]", dest, source);
        } else {
            println!("mov [{}], {}", source, dest);
        }
    } else if mode == 0b01 {
        assert!(size == 3);
        let source = EACS[regormem as usize];
        let dest = retrieve_register(reg, w).expect("Failed to get source register");
        let displacement = bytes[2] as i8;
        if d == 1 {
            println!("mov {}, [{} + {}]", dest, source, displacement);
        } else {
            println!("mov [{} + {}], {}", source, displacement, dest);
        }
    } else if mode == 0b10 {
        assert!(size == 4);
        let source = EACS[regormem as usize];
        let dest = retrieve_register(reg, w).expect("Failed to get source register");
        let displacement = ((bytes[3] as i16) << 8) | (bytes[2] as i16);
        if d == 1 {
            println!("mov {}, [{} + {}]", dest, source, displacement);
        } else {
            println!("mov [{} + {}], {}", source, displacement, dest);
        }
    } else {
        let (source, destination) = if d == 0 {
            // MOV from register/memory to register
            (
                retrieve_register(regormem, w).expect("Failed to get source register"),
                retrieve_register(reg, w).expect("Failed to get destination register"),
            )
        } else {
            // MOV from register to register/memory
            (
                retrieve_register(reg, w).expect("Failed to get source register"),
                retrieve_register(regormem, w).expect("Failed to get destination register"),
            )
        };

        println!("mov {}, {}", source, destination);
    }
}
