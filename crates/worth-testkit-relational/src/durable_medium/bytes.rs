//! Concrete byte storage shared by both Native test compilation identities.
use std::collections::BTreeMap;
use worth_execution::ExecutionImmutableBytes;

#[derive(Debug, Default)]
pub struct ByteJournal {
    pub entries: BTreeMap<u64, Box<[u8]>>,
}
impl ByteJournal {
    pub fn head(&self) -> u64 {
        self.entries.keys().next_back().copied().unwrap_or(0)
    }
    /// The original fixture rule: append next, or rejoin identical stored bytes.
    pub fn publish(&mut self, position: u64, bytes: &[u8]) -> bool {
        if self.head().checked_add(1) == Some(position) {
            self.entries.insert(position, Box::from(bytes));
            true
        } else {
            self.entries.get(&position).map(AsRef::as_ref) == Some(bytes)
        }
    }
}

#[derive(Debug, Default)]
pub struct MemberBytes {
    entries: BTreeMap<([u8; 32], u64), ExecutionImmutableBytes>,
}
impl MemberBytes {
    pub fn insert(&mut self, digest: [u8; 32], length: u64, bytes: ExecutionImmutableBytes) {
        self.entries.insert((digest, length), bytes);
    }
    pub fn iter(&self) -> impl Iterator<Item = (&([u8; 32], u64), &[u8])> {
        self.entries.iter().map(|(key, bytes)| (key, bytes.bytes()))
    }
    pub fn get(&self, digest: [u8; 32], length: u64) -> Option<&[u8]> {
        self.entries.get(&(digest, length)).map(AsRef::as_ref)
    }
}
