//! Authenticated dictionary page routing and the recorded identity frontier.
use super::{
    checkpoint_pages::CheckpointPages, checkpoint_symbol_buckets::CheckpointSymbolBuckets,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use serde::{Deserialize, Serialize};

/// Value lookup has its own directory so interning a new value need not load
/// the id-ordered dictionary. Successive publications append immutable pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointSymbolBucket {
    pub bucket: u8,
    pub pages: Vec<RelationalDurableMemberLocator>,
}

#[derive(Clone, Copy)]
pub struct SymbolPageRoster<'a> {
    pub by_id: &'a CheckpointPages,
    pub by_value: &'a CheckpointSymbolBuckets,
    pub frontier: u32,
}
