use crate::utility::BitTrie;
use once_cell::sync::Lazy;
use std::fmt;

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

pub static OPCODE_TRIE: Lazy<BitTrie> = Lazy::new(|| {
    let mut trie = BitTrie::default();
    trie.insert(0b100010, 6, Opcode::MovRmR);
    trie.insert(0b1011, 4, Opcode::MovIR);
    trie.insert(0b1100011, 7, Opcode::MovIRm);
    trie.insert(0b1010001, 7, Opcode::MovAM);
    trie.insert(0b1010000, 7, Opcode::MovMA);
    trie.insert(0b000000, 6, Opcode::AddRmR);
    trie.insert(0b100000, 6, Opcode::AddIRm);
    trie.insert(0b0000010, 7, Opcode::AddIA);
    trie
});
