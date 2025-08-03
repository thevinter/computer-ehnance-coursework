use core::panic;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::fs::{self};
use std::io::{self, Read};
use std::path::Path;

static DEBUG: bool = false;

#[derive(Copy, Clone, Debug)]
enum Opcode {
    MovRmR, // Register or Memory to Register
    MovIR,  // Immediate to Register
    MovIRm, // Immediate to Register or Memory
    MovAM,  // Accumulator to Memory
    MovMA,  // Memory to Accumulator
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::MovRmR | Opcode::MovIR | Opcode::MovIRm | Opcode::MovAM | Opcode::MovMA => {
                write!(f, "mov")
            }
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
    trie.insert(0b1100011, 7, Opcode::MovIRm);
    trie.insert(0b1010001, 7, Opcode::MovAM);
    trie.insert(0b1010000, 7, Opcode::MovMA);
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

trait IteratorExt: Iterator<Item = u8> {
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
                eprintln!("Unknown opcode: {:08b}", b0);
                std::process::exit(1);
            });

        match opcode {
            Opcode::MovMA => {
                let mut bytes = vec![b0];

                bytes.push(reader.next_or_exit("mov m-a addr_lo"));
                bytes.push(reader.next_or_exit("mov m-a addr_hi"));

                process_mov_ma(&bytes, bytes.len() as u8);
            }
            Opcode::MovAM => {
                let mut bytes = vec![b0];

                bytes.push(reader.next_or_exit("mov a-m addr_lo"));
                bytes.push(reader.next_or_exit("mov a-m addr_hi"));

                process_mov_am(&bytes, bytes.len() as u8);
            }
            Opcode::MovRmR => {
                let mut bytes = vec![b0];
                bytes.push(reader.next_or_exit("mov rm-r b[1]"));

                let mode = bytes[1] >> 6;
                let displacement_registers = if mode == 0b11 {
                    0
                } else {
                    // Special case for 16-bit displacement
                    if mode == 0b00 && (bytes[1] & 0b111) == 0b110 {
                        2
                    } else {
                        mode
                    }
                };

                for _ in 0..displacement_registers {
                    bytes.push(reader.next_or_exit("mov rm-r displ"));
                }

                process_mov_rmr(&bytes, bytes.len() as u8);
            }
            Opcode::MovIR => {
                let w = (b0 >> 3) & 0b1;
                let mut bytes = vec![b0];

                for _ in 0..w + 1 {
                    bytes.push(reader.next_or_exit("mov i-r data"));
                }

                process_mov_ir(&bytes, bytes.len() as u8);
            }
            Opcode::MovIRm => {
                let w = b0 & 0b1;
                let mut bytes = vec![b0];
                bytes.push(reader.next_or_exit("mov i-rm b[1]"));

                let mode = bytes[1] >> 6;
                let displacement_registers = if mode == 0b11 { 0 } else { mode };

                for _ in 0..displacement_registers {
                    bytes.push(reader.next_or_exit("mov i-rm displ"));
                }

                for _ in 0..w + 1 {
                    bytes.push(reader.next_or_exit("mov i-rm data"));
                }

                process_mov_irm(&bytes, bytes.len() as u8);
            }
        };
    }
}

fn debug_bytes(bytes: &[u8]) {
    if DEBUG {
        println!(
            "Processing bytes: [{}]",
            bytes
                .iter()
                .map(|b| format!("{:08b}", b))
                .collect::<Vec<_>>()
                .join(", ")
        );
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

// Memory to Accumulator
fn process_mov_ma(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov ma: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov ax, [{}]", value);
}

// Accumulator to Memory
fn process_mov_am(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov am: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov [{}], ax", value);
}

// Immediate to Register or Memory
fn process_mov_irm(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size >= 3 && size <= 6, "Invalid size for mov irm: {}", size);

    let w = (bytes[0] >> 3) & 0b1;
    let reg = bytes[0] & 0b111;
    let mode = bytes[1] >> 6;

    let regormem = bytes[1] & 0b111;
    match mode {
        0b00 => {
            // Immediate to memory
            assert!(
                size == 4 || size == 3,
                "Invalid size for mov irm mode im to mem: {}",
                size
            );
            let dest = EACS[regormem as usize];
            let value = if size == 3 {
                let _b = bytes[2] as i8;
                _b as i16
            } else {
                ((bytes[3] as i16) << 8) | (bytes[2] as i16)
            };
            let size = if size == 3 { "byte" } else { "word" };
            println!("mov [{}], {} {}", dest, size, value);
        }

        0b01 => {
            // Immediate to memory with 8-bit displacement
            assert!(
                size == 4,
                "Invalid size for mov irm mode im to mem + 8b: {}",
                size
            );
            let dest = EACS[regormem as usize];
            let displacement = bytes[2] as i8;
            let value = bytes[3] as i8;
            println!(
                "mov [{} {}], word {}",
                dest,
                with_sign(displacement as i16),
                value
            );
        }

        0b10 => {
            // Immediate to memory with 16-bit displacement
            assert!(
                size == 6,
                "Invalid size for mov irm mode im to mem + 16b: {}",
                size
            );
            let dest = EACS[regormem as usize];
            let displacement = ((bytes[3] as i16) << 8) | (bytes[2] as i16);
            let value = ((bytes[5] as i16) << 8) | (bytes[4] as i16);
            println!("mov [{} {}], word {}", dest, with_sign(displacement), value);
        }

        0b11 => {
            // Immediate to register
            assert!(
                size == 3,
                "Invalid size for mov irm mode im to reg: {}",
                size
            );
            let dest = retrieve_register(reg, w).expect("Failed to get destination register");
            let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
            println!("mov {}, {}", dest, value);
        }

        _ => {
            panic!("Unsupported mode: {:02b}", mode);
        }
    }
}

fn process_mov_ir(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3 || size == 4, "Invalid size for mov ir {}", size);

    let w = (bytes[0] >> 3) & 0b1;
    let reg = bytes[0] & 0b111;
    let value = if size == 3 {
        ((bytes[2] as i16) << 8) | (bytes[1] as i16)
    } else {
        let _b = bytes[1] as i8;
        _b as i16
    };

    let dest = retrieve_register(reg, w).expect("Failed to get destination register");

    println!("mov {}, {}", dest, value);
}

fn process_mov_rmr(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size >= 2 && size <= 4, "Invalid size for mov rmr: {}", size);

    let d = (bytes[0] >> 1) & 0b1;
    let w = bytes[0] & 0b1;

    let mode = bytes[1] >> 6;
    let reg = (bytes[1] >> 3) & 0b111;
    let regormem = bytes[1] & 0b111;

    match mode {
        0b00 => {
            let source = if regormem == 0b110 {
                (((bytes[3] as i16) << 8) | (bytes[2] as i16)).to_string()
            } else {
                EACS[regormem as usize].to_string()
            };
            let dest = retrieve_register(reg, w).expect("Failed to get source register");
            if d == 1 {
                println!("mov {}, [{}]", dest, source);
            } else {
                println!("mov [{}], {}", source, dest);
            }
        }

        0b01 => {
            assert!(size == 3);
            let source = EACS[regormem as usize];
            let dest = retrieve_register(reg, w).expect("Failed to get source register");
            let _disp = bytes[2] as i8;
            let displacement = _disp as i16;
            if d == 1 {
                println!("mov {}, [{} {}]", dest, source, with_sign(displacement));
            } else {
                println!("mov [{} {}], {}", source, with_sign(displacement), dest);
            }
        }

        0b10 => {
            assert!(size == 4);
            let source = EACS[regormem as usize];
            let dest = retrieve_register(reg, w).expect("Failed to get source register");
            let displacement = ((bytes[3] as i16) << 8) | (bytes[2] as i16);
            if d == 1 {
                println!("mov {}, [{} {}]", dest, source, with_sign(displacement));
            } else {
                println!("mov [{} {}], {}", source, with_sign(displacement), dest);
            }
        }

        0b11 => {
            let (source, destination) = if d == 0 {
                (
                    retrieve_register(regormem, w).expect("Failed to get source register"),
                    retrieve_register(reg, w).expect("Failed to get destination register"),
                )
            } else {
                (
                    retrieve_register(reg, w).expect("Failed to get source register"),
                    retrieve_register(regormem, w).expect("Failed to get destination register"),
                )
            };
            println!("mov {}, {}", source, destination);
        }

        _ => panic!("Invalid mode: {:02b}", mode),
    }
}

fn with_sign(n: i16) -> String {
    if n >= 0 {
        format!("+ {}", n)
    } else {
        format!("- {}", -n)
    }
}
