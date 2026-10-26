//! Authenticated page contents must also match their selected directory entry.
use crate::durability::checkpoints::symbol_pages::SYMBOL_PAGE_CAPACITY;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailableCause as Cause};
use crate::symbols::data::{Symbol, SymbolTableSnapshot};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub(super) enum SymbolPageSelection {
    IdRange { first: Symbol, last: Symbol },
    HashBucket(u8),
}

impl SymbolPageSelection {
    pub(super) fn subject(self) -> RelationalStorageSubject {
        let (first, last) = match self {
            Self::IdRange { first, last } => (first, last),
            Self::HashBucket(_) => (Symbol(0), Symbol(u32::MAX - 1)),
        };
        RelationalStorageSubject::Symbols { first, last }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SymbolPageShape {
    first: Symbol,
    last: Symbol,
    homogeneous_bucket: Option<u8>,
}

impl SymbolPageShape {
    pub(super) fn read(snapshot: &SymbolTableSnapshot) -> Result<Self, Cause> {
        if snapshot.entries.is_empty()
            || snapshot.entries.len() > SYMBOL_PAGE_CAPACITY
            || snapshot
                .entries
                .windows(2)
                .any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(Cause::Damaged);
        }
        let first = &snapshot.entries[0];
        let bucket = Sha256::digest(first.1.as_str().as_bytes())[0];
        Ok(Self {
            first: first.0,
            last: snapshot.entries.last().unwrap().0,
            homogeneous_bucket: snapshot
                .entries
                .iter()
                .all(|(_, text)| Sha256::digest(text.as_str().as_bytes())[0] == bucket)
                .then_some(bucket),
        })
    }

    pub(super) fn validate(self, selection: SymbolPageSelection) -> Result<(), Cause> {
        let matches = match selection {
            SymbolPageSelection::IdRange { first, last } => {
                self.first == first && self.last == last
            }
            SymbolPageSelection::HashBucket(bucket) => self.homogeneous_bucket == Some(bucket),
        };
        if matches {
            Ok(())
        } else {
            Err(Cause::Damaged)
        }
    }
}
