use crate::utility::BitTrie;
use once_cell::sync::Lazy;
use std::fmt::{self, write};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    MovRmR, // Register or Memory to Register
    MovIR,  // Immediate to Register
    MovIRm, // Immediate to Register or Memory
    MovAM,  // Accumulator to Memory
    MovMA,  // Memory to Accumulator
    AddRmR,
    AddIA,
    AddIRm,
    SubRmR,
    SubIA,
    SubIRm,
    CmpRmR,
    CmpIRm,
    CmpIA,
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jg,
    Jnb,
    Ja,
    Jnp,
    Jno,
    Jns,
    Loop,
    Loopz,
    Loopnz,
    Jcxz,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::MovRmR | Opcode::MovIR | Opcode::MovIRm | Opcode::MovAM | Opcode::MovMA => {
                write!(f, "mov")
            }
            Opcode::AddRmR | Opcode::AddIRm | Opcode::AddIA => write!(f, "add"),
            Opcode::SubRmR | Opcode::SubIRm | Opcode::SubIA => write!(f, "sub"),
            Opcode::CmpRmR | Opcode::CmpIRm | Opcode::CmpIA => write!(f, "cmp"),
            Opcode::Je => write!(f, "je"),
            Opcode::Jl => write!(f, "jl"),
            Opcode::Jle => write!(f, "jle"),
            Opcode::Jb => write!(f, "jb"),
            Opcode::Jbe => write!(f, "jbe"),
            Opcode::Jp => write!(f, "jp"),
            Opcode::Jo => write!(f, "jo"),
            Opcode::Js => write!(f, "js"),
            Opcode::Jne => write!(f, "jne"),
            Opcode::Jnl => write!(f, "jnl"),
            Opcode::Jg => write!(f, "jg"),
            Opcode::Jnb => write!(f, "jnb"),
            Opcode::Ja => write!(f, "ja"),
            Opcode::Jnp => write!(f, "jnp"),
            Opcode::Jno => write!(f, "jno"),
            Opcode::Jns => write!(f, "jns"),
            Opcode::Loop => write!(f, "loop"),
            Opcode::Loopz => write!(f, "loopz"),
            Opcode::Loopnz => write!(f, "loopnz"),
            Opcode::Jcxz => write!(f, "jcxz"),
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
    trie.insert(0b001010, 6, Opcode::SubRmR);
    trie.insert(0b0010110, 7, Opcode::SubIA);
    trie.insert(0b001110, 6, Opcode::CmpRmR);
    trie.insert(0b0011110, 7, Opcode::CmpIA);
    trie.insert(0b01110100, 8, Opcode::Je);
    trie.insert(0b01111100, 8, Opcode::Jl);
    trie.insert(0b01111110, 8, Opcode::Jle);
    trie.insert(0b01110010, 8, Opcode::Jb);
    trie.insert(0b01110110, 8, Opcode::Jbe);
    trie.insert(0b01111010, 8, Opcode::Jp);
    trie.insert(0b01110000, 8, Opcode::Jo);
    trie.insert(0b01111000, 8, Opcode::Js);
    trie.insert(0b01110101, 8, Opcode::Jne);
    trie.insert(0b01111101, 8, Opcode::Jnl);
    trie.insert(0b01111111, 8, Opcode::Jg);
    trie.insert(0b01110011, 8, Opcode::Jnb);
    trie.insert(0b01110111, 8, Opcode::Ja);
    trie.insert(0b01111011, 8, Opcode::Jnp);
    trie.insert(0b01110001, 8, Opcode::Jno);
    trie.insert(0b01111001, 8, Opcode::Jns);
    trie.insert(0b11100010, 8, Opcode::Loop);
    trie.insert(0b11100001, 8, Opcode::Loopz);
    trie.insert(0b11100000, 8, Opcode::Loopnz);
    trie.insert(0b11100011, 8, Opcode::Jcxz);
    trie
});
