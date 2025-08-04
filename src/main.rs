use core::panic;
use std::env;
use std::fs::{self};
use std::io::{self};

mod opcodes;
mod registers;
mod utility;

use opcodes::{Opcode, OPCODE_TRIE};
use registers::{retrieve_register, Register, EAC, EACS, REGISTERS};
use utility::{debug_bytes, read_file, BitTrie, IteratorExt, Reader};

static DEBUG: bool = true;

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

                process_rmr(&bytes, bytes.len() as u8, Opcode::MovRmR);
            }
            Opcode::MovIR => {
                let w = (b0 >> 3) & 0b1;
                let mut bytes = vec![b0];

                for _ in 0..w + 1 {
                    bytes.push(reader.next_or_exit("mov i-r data"));
                }

                process_ir(&bytes, bytes.len() as u8, Opcode::MovIR);
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

                process_irm(&bytes, bytes.len() as u8, Opcode::MovIRm);
            }
            Opcode::AddRmR => {
                let mut bytes = vec![b0];
                bytes.push(reader.next_or_exit("add rm-r b[1]"));

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
                    bytes.push(reader.next_or_exit("add rm-r displ"));
                }

                process_rmr(&bytes, bytes.len() as u8, Opcode::AddRmR);
            }
            Opcode::AddIRm => {
                let w = b0 & 0b1;
                let s = (b0 >> 1) & 0b1;
                let mut bytes = vec![b0];
                bytes.push(reader.next_or_exit("add i-rm b[1]"));

                let mode = bytes[1] >> 6;
                let displacement_registers = if mode == 0b11 { 0 } else { mode };

                for _ in 0..displacement_registers {
                    bytes.push(reader.next_or_exit("add i-rm displ"));
                }

                for _ in 0..(w + 1 - s) {
                    bytes.push(reader.next_or_exit("add i-rm data"));
                }

                process_irm(&bytes, bytes.len() as u8, Opcode::AddIRm);
            }
            Opcode::AddIA => {
                let w = b0 & 0b1;
                let mut bytes = vec![b0];

                for _ in 0..w + 1 {
                    bytes.push(reader.next_or_exit("mov i-r data"));
                }

                process_ir(&bytes, bytes.len() as u8, Opcode::AddIA);
            }
        };
    }
}

fn process_irm(bytes: &[u8], size: u8, op: Opcode) {
    debug_bytes(bytes);
    assert!(
        size >= 3 && size <= 6,
        "Invalid size for {} irm: {}",
        op,
        size
    );

    let w = bytes[0] & 0b1;
    let reg = bytes[1] & 0b111;
    let mode = bytes[1] >> 6;

    let regormem = bytes[1] & 0b111;
    match mode {
        0b00 => {
            // Immediate to memory
            assert!(
                size == 4 || size == 3,
                "Invalid size for {} irm mode im to mem: {}",
                op,
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
            println!("{} [{}], {} {}", op, dest, size, value);
        }

        0b01 => {
            // Immediate to memory with 8-bit displacement
            assert!(
                size == 4,
                "Invalid size for {} irm mode im to mem + 8b: {}",
                op,
                size
            );
            let dest = EACS[regormem as usize];
            let displacement = bytes[2] as i8;
            let value = bytes[3] as i8;
            println!(
                "{} [{} {}], word {}",
                op,
                dest,
                with_sign(displacement as i16),
                value
            );
        }

        0b10 => {
            // Immediate to memory with 16-bit displacement
            assert!(
                size == 6 || size == 5,
                "Invalid size for {} irm mode im to mem + 16b: {}",
                op,
                size
            );
            let dest = EACS[regormem as usize];
            let displacement = ((bytes[3] as i16) << 8) | (bytes[2] as i16);
            let value = if op == Opcode::AddIRm {
                bytes[4] as i16
            } else {
                ((bytes[5] as i16) << 8) | (bytes[4] as i16)
            };
            println!(
                "{} [{} {}], word {}",
                op,
                dest,
                with_sign(displacement),
                value
            );
        }

        0b11 => {
            // Immediate to register
            assert!(
                size == 3 || size == 4,
                "Invalid size for {} irm mode im to reg: {}",
                op,
                size
            );
            let dest = retrieve_register(reg, w).expect("Failed to get destination register");
            let value = if op == Opcode::AddIRm {
                bytes[2] as i16
            } else {
                ((bytes[2] as i16) << 8) | (bytes[1] as i16)
            };
            println!("{} {}, {}", op, dest, value);
        }

        _ => {
            panic!("Unsupported mode: {:02b}", mode);
        }
    }
}

fn process_rmr(bytes: &[u8], size: u8, op: Opcode) {
    debug_bytes(bytes);
    assert!(
        size >= 2 && size <= 4,
        "Invalid size for {} rmr: {}",
        op,
        size
    );

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
                println!("{} {}, [{}]", op, dest, source);
            } else {
                println!("{} [{}], {}", op, source, dest);
            }
        }

        0b01 => {
            assert!(size == 3);
            let source = EACS[regormem as usize];
            let dest = retrieve_register(reg, w).expect("Failed to get source register");
            let _disp = bytes[2] as i8;
            let displacement = _disp as i16;
            if d == 1 {
                println!("{} {}, [{} {}]", op, dest, source, with_sign(displacement));
            } else {
                println!("{} [{} {}], {}", op, source, with_sign(displacement), dest);
            }
        }

        0b10 => {
            assert!(size == 4);
            let source = EACS[regormem as usize];
            let dest = retrieve_register(reg, w).expect("Failed to get source register");
            let displacement = ((bytes[3] as i16) << 8) | (bytes[2] as i16);
            if d == 1 {
                println!("{} {}, [{} {}]", op, dest, source, with_sign(displacement));
            } else {
                println!("{} [{} {}], {}", op, source, with_sign(displacement), dest);
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
            println!("{} {}, {}", op, source, destination);
        }

        _ => panic!("Invalid mode: {:02b}", mode),
    }
}

fn process_mov_ma(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov ma: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov ax, [{}]", value);
}

fn process_mov_am(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov am: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov [{}], ax", value);
}

fn process_ir(bytes: &[u8], size: u8, op: Opcode) {
    debug_bytes(bytes);
    assert!(
        size == 3 || size == 2,
        "Invalid size for {} ir {}",
        op,
        size
    );

    let w = if (op == Opcode::AddIA) {
        bytes[0] & 0b1
    } else {
        (bytes[0] >> 3) & 0b1
    };
    let reg = bytes[0] & 0b111;
    let value = if size == 3 {
        ((bytes[2] as i16) << 8) | (bytes[1] as i16)
    } else {
        let _b = bytes[1] as i8;
        _b as i16
    };

    let dest = if op == Opcode::AddIA {
        if size == 3 {
            "ax"
        } else {
            "al"
        }
    } else {
        &retrieve_register(reg, w)
            .expect("Failed to get destination register")
            .to_string()
    };

    println!("{} {}, {}", op, dest, value);
}
fn process_mov_ir(bytes: &[u8], size: u8) {
    debug_bytes(bytes);
    assert!(size == 3 || size == 2, "Invalid size for mov ir {}", size);

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

fn with_sign(n: i16) -> String {
    if n >= 0 {
        format!("+ {}", n)
    } else {
        format!("- {}", -n)
    }
}
