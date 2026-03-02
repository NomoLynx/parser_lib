use crate::common::*;

use pest::iterators::Pair;
use pest_derive::Parser;
use rust_macro::*;

use crate::common::debug::*;

#[derive(Parser)]
#[grammar = "mermaid_packet/packet.pest"]
pub (crate) struct PackatFileParser;

/// Represents a packet diagram
#[derive(Debug, Clone, Accessors, GetMut)]
pub struct PacketSection {
    /// The name of the packet diagram
    name: String,
    /// An optional second name for the packet diagram
    name2: Option<String>,
    /// The entries in the packet diagram
    entries: Vec<PacketEntry>,
    /// The location of the packet diagram in the source, used for error reporting and debugging
    location : Location,
}

impl PacketSection {
    pub fn from_pair(pair: &Pair<Rule>) -> Option<Self> {
        assert!(pair.as_rule() == Rule::packet_diagram);
        let inner = pair.to_owned().into_inner();
        let mut name: Option<String> = None;
        let mut name2: Option<String> = None;
        let mut entries: Vec<PacketEntry> = Vec::new();
        for p in inner {
            match p.as_rule() {
                Rule::title => {
                    let title_str = p.as_str().trim();
                    if name.is_none() {
                        name = Some(title_str.to_string());
                    }
                    else if name2.is_none() {
                        name2 = Some(title_str.to_string());
                    }
                }
                Rule::field => {
                    if let Some(entry) = PacketEntry::from_pair(&p) {
                        entries.push(entry);
                    }
                }
                Rule::formatter | Rule::packet_start | Rule::NEWLINE | Rule::EOI => {
                    // ignore
                }
                _ => {
                    error_string(format!("Unexpected rule in packet_diagram: {:?}", p.as_rule()));
                }
            }
        }
        let name = name.unwrap_or_else(|| "Unnamed".to_string());

        let location = pair.into();
        Some(PacketSection::new(name, name2, entries, location))
    }

    pub fn new(name: String, name2: Option<String>, entries: Vec<PacketEntry>, location: Location) -> Self {
        Self { name, name2, entries, location, }
    }

    pub fn add_entry(&mut self, entry: PacketEntry) {
        self.entries.push(entry);
    }

    /// Get total bit size of the packet diagram
    pub fn get_total_bit_size(&self) -> u32 {
        self.entries.iter().map(|e| e.get_bit_size() as u32).sum()
    }

    /// Get total byte size of the packet diagram
    pub fn get_total_byte_size(&self) -> u32 {
        let total_bits = self.get_total_bit_size();
        (total_bits + 7) / 8
    }
}

#[derive(Debug, Clone, Accessors)]
pub struct PacketEntry {
    bit_spec: BitSpec,
    name: String,
    location: Location,
}

impl PacketEntry {
    pub fn new(bit_spec: BitSpec, name: String, location : Location) -> Self {
        Self { bit_spec, name, location }
    }

    pub fn from_pair(pair: &Pair<Rule>) -> Option<Self> {
        assert!(pair.as_rule() == Rule::field);
        let mut inner = pair.to_owned().into_inner();
        let bit_spec_pair = inner.next()?;
        let name_pair = inner.next()?;
        let bit_spec = BitSpec::from_pair(&bit_spec_pair)?;
        let name = name_pair.into_inner()
                                .next().unwrap()
                                .as_str().trim().to_string();
        let location = pair.into();
        Some(PacketEntry::new(bit_spec, name, location))
    }

    /// Get bit size of the packet entry
    pub fn get_bit_size(&self) -> u8 {
        self.bit_spec.get_bit_size()
    }

    /// get byte size of the packet entry
    pub fn get_byte_size(&self) -> u8 {
        let bits = self.get_bit_size() as u16;
        ((bits + 7) / 8) as u8
    }
}

#[derive(Debug, Clone)]
pub enum BitSpec {
    RelativeBits(i32), // +n
    BitRange(i32, i32), // n-m
    SingleBit(i32), // n
}

impl BitSpec {
    pub fn from_pair(pair: &Pair<Rule>) -> Option<Self> {
        assert!(pair.as_rule() == Rule::bit_specification);
        let inner = pair.to_owned().into_inner();
        let pairs = inner.map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::relative_bits, bit_pair)] => {
                let s = bit_pair.as_str();
                let n: i32 = s[1..].parse().ok()?;
                Some(BitSpec::RelativeBits(n))
            }
            [(Rule::bit_range, bit_pair)] => {
                let s = bit_pair.as_str();
                let parts: Vec<&str> = s.split('-').collect();
                if parts.len() != 2 {
                    return None;
                }
                let start: i32 = parts[0].parse().ok()?;
                let end: i32 = parts[1].parse().ok()?;
                Some(BitSpec::BitRange(start, end))
            }
            [(Rule::single_bit, bit_pair)] => {
                let s = bit_pair.as_str();
                let n: i32 = s.parse().ok()?;
                Some(BitSpec::SingleBit(n))
            }
            _ => None,
        }
    }

    pub fn get_bit_size(&self) -> u8 {
        match self {
            BitSpec::RelativeBits(n) => *n as u8,
            BitSpec::BitRange(start, end) => (end - start + 1) as u8,
            BitSpec::SingleBit(_) => 1,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            BitSpec::RelativeBits(n) => format!("+{}", n),
            BitSpec::BitRange(start, end) => format!("{}-{}", start, end),
            BitSpec::SingleBit(n) => format!("{}", n),
        }
    }
}