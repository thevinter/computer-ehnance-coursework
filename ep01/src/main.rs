use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::{self};

#[derive(Copy, Clone, Debug)]
enum Opcode {
    MOV,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::MOV => write!(f, "mov"),
        }
    }
}

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

static OPCODE_MAP: Lazy<HashMap<u8, Opcode>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0b100010, Opcode::MOV);
    m
});

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

    println!("File read successfully, size: {} bytes", file.len());

    for chunk in file.chunks(2) {
        if chunk.len() < 2 {
            eprintln!("Incomplete opcode chunk: {:?}", chunk);
            std::process::exit(1);
        }
        let b1 = chunk[0];
        let b2 = chunk[1];

        let opcode = b1 >> 2;
        let d = (b1 >> 1) & 0b1;
        let w = b1 & 0b1;

        let _mod = b2 >> 6;
        let reg = (b2 >> 3) & 0b111;
        let regormem = b2 & 0b111;

        match parse_opcode(opcode) {
            Ok(opcode) => {
                // println!("Parsed opcode: {:?}", opcode);
                match opcode {
                    Opcode::MOV => process_mov(d, w, reg, regormem),
                };
            }
            Err(e) => eprintln!("Error parsing opcode: {}", e),
        }
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

fn parse_opcode(buffer: u8) -> Result<Opcode, String> {
    OPCODE_MAP
        .get(&buffer)
        .copied()
        .ok_or_else(|| format!("Unknown opcode: {}", buffer).into())
}

fn process_mov(d: u8, w: u8, reg: u8, regormem: u8) {
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
    println!("mov {} {}", source, destination);
}
