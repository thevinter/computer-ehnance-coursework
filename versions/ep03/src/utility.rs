use crate::opcodes::Opcode;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

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

pub fn debug_bytes(bytes: &[u8]) {
    static DEBUG: bool = true;
    if DEBUG {
        println!(
            "; Processing bytes: [{}]",
            bytes
                .iter()
                .map(|b| format!("{:08b}", b))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    use std::fs;
    fs::read(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}
