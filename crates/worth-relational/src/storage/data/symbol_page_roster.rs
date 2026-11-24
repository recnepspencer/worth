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
pub(crate) struct RelationalCheckpointSymbolBucket {
    pub(crate) bucket: u8,
    pub(crate) pages: Vec<RelationalDurableMemberLocator>,
}

#[derive(Clone, Copy)]
pub(crate) struct SymbolPageRoster<'a> {
    pub(crate) by_id: &'a CheckpointPages,
    pub(crate) by_value: &'a CheckpointSymbolBuckets,
    pub(crate) frontier: u32,
}
