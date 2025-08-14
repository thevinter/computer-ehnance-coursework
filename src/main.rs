use core::panic;
use std::env;

mod opcodes;
mod registers;
mod utility;

use opcodes::{OPCODE_TRIE, Opcode};
use registers::{EACS, RegisterFile, retrieve_register};
use utility::{DEBUG, debug_bytes, read_file};

use crate::registers::Register;
use crate::utility::{print_memory_16bit, print_memory_hex};

macro_rules! debug {
    ($($arg:tt)*) => {
        if utility::DEBUG {
            println!($($arg)*);
        }
    };
}

fn main() {
    let mut args = env::args();

    args.next();

    let mut memory = RegisterFile::new();
    print_memory_16bit(&memory.raw_memory());

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

    println!("; File read successfully, size: {} bytes", file.len());

    fn read_or_exit(file: &[u8], index: usize, context: &str) -> u8 {
        *file.get(index).unwrap_or_else(|| {
            eprintln!("Unexpected end of file while reading {}", context);
            std::process::exit(1);
        })
    }

    loop {
        let ip = memory.get(Register::IP) as usize;
        if ip >= file.len() {
            break;
        }

        let b0 = read_or_exit(&file, ip, "opcode");

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

                bytes.push(read_or_exit(&file, ip + bytes.len(), "mov m-a addr_lo"));
                bytes.push(read_or_exit(&file, ip + bytes.len(), "mov m-a addr_hi"));

                process_mov_ma(&bytes, bytes.len() as u8, &mut memory);
            }
            Opcode::MovAM => {
                let mut bytes = vec![b0];

                bytes.push(read_or_exit(&file, ip + bytes.len(), "mov a-m addr_lo"));
                bytes.push(read_or_exit(&file, ip + bytes.len(), "mov a-m addr_hi"));

                process_mov_am(&bytes, bytes.len() as u8, &mut memory);
            }
            Opcode::MovIR => {
                let w = (b0 >> 3) & 0b1;
                let mut bytes = vec![b0];

                for _ in 0..w + 1 {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "mov i-r data"));
                }

                process_ir(&bytes, bytes.len() as u8, Opcode::MovIR, &mut memory);
            }
            Opcode::MovIRm => {
                let w = b0 & 0b1;
                let mut bytes = vec![b0];
                bytes.push(read_or_exit(&file, ip + bytes.len(), "mov i-rm b[1]"));

                let mode = bytes[1] >> 6;
                let displacement_registers = if mode == 0b11 { 0 } else { mode };

                for _ in 0..displacement_registers {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "mov i-rm displ"));
                }

                for _ in 0..w + 1 {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "mov i-rm data"));
                }

                process_irm(&bytes, bytes.len() as u8, Opcode::MovIRm, &mut memory);
            }
            Opcode::AddIRm | Opcode::SubIRm | Opcode::CmpIRm => {
                let w = b0 & 0b1;
                let s = (b0 >> 1) & 0b1;
                let mut bytes = vec![b0];
                bytes.push(read_or_exit(&file, ip + bytes.len(), "add i-rm b[1]"));

                let regormem = bytes[1] & 0b111;

                let mode = bytes[1] >> 6;
                let displacement_registers = if mode == 0b11 {
                    0
                } else {
                    if regormem == 0b110 { 2 } else { mode }
                };

                for _ in 0..displacement_registers {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "add i-rm displ"));
                }

                for _ in 0..(w + 1 - s) {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "add i-rm data"));
                }

                process_irm(&bytes, bytes.len() as u8, opcode, &mut memory);
            }
            Opcode::AddIA | Opcode::SubIA | Opcode::CmpIA => {
                let w = b0 & 0b1;
                let mut bytes = vec![b0];

                for _ in 0..w + 1 {
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "mov i-r data"));
                }

                process_ir(&bytes, bytes.len() as u8, opcode, &mut memory);
            }
            Opcode::SubRmR | Opcode::AddRmR | Opcode::CmpRmR | Opcode::MovRmR => {
                let mut bytes = vec![b0];
                bytes.push(read_or_exit(&file, ip + bytes.len(), "sub rm-r b[1]"));

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
                    bytes.push(read_or_exit(&file, ip + bytes.len(), "{} rm-r displ"));
                }

                process_rmr(&bytes, bytes.len() as u8, opcode, &mut memory);
            }
            Opcode::Je
            | Opcode::Jl
            | Opcode::Jle
            | Opcode::Jb
            | Opcode::Jbe
            | Opcode::Jp
            | Opcode::Jo
            | Opcode::Js
            | Opcode::Jne
            | Opcode::Jnl
            | Opcode::Jg
            | Opcode::Jnb
            | Opcode::Ja
            | Opcode::Jnp
            | Opcode::Jno
            | Opcode::Jns
            | Opcode::Loop
            | Opcode::Loopz
            | Opcode::Loopnz
            | Opcode::Jcxz => {
                let mut bytes = vec![b0];

                bytes.push(read_or_exit(&file, ip + bytes.len(), "jmp ip-inc8"));

                process_jmp(&bytes, bytes.len() as u8, opcode, &mut memory);
            }
        };
        println!();
    }
}

fn process_irm(bytes: &[u8], size: u8, op: Opcode, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug!("; Processing irm:");
    debug_bytes(bytes);
    assert!(
        size >= 3 && size <= 6,
        "Invalid size for {} irm: {}",
        op,
        size
    );

    let w = bytes[0] & 0b1;
    let reg = (bytes[1] >> 3) & 0b111;
    let mode = bytes[1] >> 6;
    let regormem = bytes[1] & 0b111;

    let is_arithmetic = op == Opcode::AddIRm || op == Opcode::SubIRm || op == Opcode::CmpIRm;
    let s = if is_arithmetic {
        (bytes[0] >> 1) & 0b1
    } else {
        0
    };

    //println!("reg {:03b}, opcode: {}", reg, op);
    let op = if op == Opcode::AddIRm {
        match reg {
            0b101 => Opcode::SubIRm,
            0b000 => Opcode::AddIRm,
            0b111 => Opcode::CmpIRm,
            _ => panic!("Invalid reg for AddRmR: {}", regormem),
        }
    } else {
        op
    };
    debug!(
        "; Processing {} with mode {:02b}, reg: {:03b}, regormem: {:03b}, w = {}, s = {}",
        op, mode, reg, regormem, w, s
    );

    match mode {
        0b00 => {
            // Immediate to memory
            assert!(
                size == 4 || size == 3 || size == 5,
                "Invalid size for {} irm mode im to mem: {}",
                op,
                size
            );
            let value = if size == 3 {
                let _b = bytes[2] as i8;
                _b as i16
            } else {
                ((bytes[3] as i16) << 8) | (bytes[2] as i16)
            };
            let dest = if regormem == 0b110 {
                &value.to_string()
            } else {
                &EACS[regormem as usize].to_string()
            };
            let value = if size == 3 {
                let _b = bytes[2] as i8;
                _b as i16
            } else if size == 5 {
                let _b = bytes[4] as i8;
                _b as i16
            } else {
                ((bytes[3] as i16) << 8) | (bytes[2] as i16)
            };
            let size = if size + s == 3 { "byte" } else { "word" };
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
            let value = if is_arithmetic {
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
            let dest = retrieve_register(regormem, w).expect("Failed to get destination register");

            let value = if s == 1 && w == 1 {
                bytes[2] as i8 as i16 // sign-extend 8-bit immediate
            } else if w == 0 {
                bytes[2] as i8 as i16 // also sign-extend for 8-bit ops
            } else {
                (bytes[3] as i16) << 8 | (bytes[2] as i16)
            };

            if is_arithmetic {
                perform_arithmetic(op, dest, value, memory);
            } else {
                move_data(dest, value, memory);
            }
            println!("{} {}, {}", op, dest, value);
        }

        _ => {
            panic!("Unsupported mode: {:02b}", mode);
        }
    }
}

fn process_rmr(bytes: &[u8], size: u8, op: Opcode, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug!("; Processing rmr:");
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

    let is_arithmetic = op == Opcode::AddRmR || op == Opcode::SubRmR || op == Opcode::CmpRmR;

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
            let (source, destination) = if d == 1 {
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
            if is_arithmetic {
                perform_arithmetic(op, destination, memory.get(source) as i16, memory);
            } else {
                move_data(destination, memory.get(source) as i16, memory);
            }

            println!("{} {}, {}", op, destination, source);
        }

        _ => panic!("Invalid mode: {:02b}", mode),
    }
}

fn process_mov_ma(bytes: &[u8], size: u8, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug!("; Processing mov ma:");
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov ma: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov ax, [{}]", value);
}

fn process_mov_am(bytes: &[u8], size: u8, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug!("; Processing mov am:");
    debug_bytes(bytes);
    assert!(size == 3, "Invalid size for mov am: {}", size);

    let value = ((bytes[2] as i16) << 8) | (bytes[1] as i16);
    println!("mov [{}], ax", value);
}

fn process_jmp(bytes: &[u8], size: u8, op: Opcode, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug_bytes(bytes);
    assert!(size == 2, "Invalid size for jmp: {}", size);

    let value = bytes[1] as i8;
    match op {
        Opcode::Jne => {
            if !memory.get_flag(registers::Flag::Zero) {
                memory.move_ip_by_n(value as usize);
            }
        }

        _ => {
            panic!("Unsupported jump opcode: {:?}", op);
        }
    }
    println!("{} {}", op, value);
}

fn process_ir(bytes: &[u8], size: u8, op: Opcode, memory: &mut RegisterFile) {
    memory.move_ip_by_n(bytes.len());
    debug!("; Processing ir:");
    debug_bytes(bytes);
    assert!(
        size == 3 || size == 2,
        "Invalid size for {} ir {}",
        op,
        size
    );

    let is_arithmetic = op == Opcode::AddIA || op == Opcode::SubIA || op == Opcode::CmpIA;

    let w = if op == Opcode::AddIA {
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

    let dest = if is_arithmetic {
        if size == 3 {
            Register::AX
        } else {
            Register::AL
        }
    } else {
        retrieve_register(reg, w).expect("Failed to get destination register")
    };

    println!("{} {}, {}", op, dest, value);
    move_data(dest, value, memory);
}

fn with_sign(n: i16) -> String {
    if n >= 0 {
        format!("+ {}", n)
    } else {
        format!("- {}", -n)
    }
}

fn move_data(dest: Register, value: i16, memory: &mut RegisterFile) {
    debug!("; Moving data: {:016b} to {}", value, dest);
    memory.set(dest, value as u16);
    if DEBUG {
        print_memory_hex(&memory.raw_memory());
    }
}

fn perform_arithmetic(op: Opcode, dest: Register, value: i16, memory: &mut RegisterFile) {
    let current_value = memory.get(dest) as i16;
    let result = match op {
        Opcode::AddRmR | Opcode::AddIA | Opcode::AddIRm => current_value + value,
        Opcode::SubRmR | Opcode::SubIA | Opcode::SubIRm => current_value - value,
        Opcode::CmpRmR | Opcode::CmpIA | Opcode::CmpIRm => current_value - value,

        _ => panic!("Unsupported arithmetic operation: {:?}", op),
    };
    memory.set_flags_from_result(result);
    if DEBUG {
        memory.print_flags();
    }
    if op != Opcode::CmpRmR {
        move_data(dest, result, memory)
    };
}
