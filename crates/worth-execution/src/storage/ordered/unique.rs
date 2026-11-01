//! Copy-only edits of the actual unique cells; persistent edits remain separate.
mod edit;
mod preflight;
#[cfg(test)]
mod tests;

use super::{node, ExecutionOrderedMap, ExecutionOrderedMapDenial};
use crate::{ExecutionAllocationPolicy, ExecutionRequest};
use std::sync::Arc;

/// A point refusal occurs before any map mutation. No snapshot is copied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionOrderedMapUniqueDenial {
    SharedStorage,
    Occupied,
    CapacityOverflow,
}

/// One detached original node, owning its entry and both original grants.
/// Non-Clone and opaque: no Arc/Weak or unfunded owned payload can escape.
/// Drop this owner outside registry gates. Preparation admits both allocations
/// before creating either one; removal moves this same storage out of the map.
pub struct ExecutionOrderedMapCell<K, V> {
    node: Arc<node::Node<K, V>>,
}
impl<K, V> ExecutionOrderedMapCell<K, V> {
    pub fn key(&self) -> &K {
        &self.node.entry.key
    }
    pub fn value(&self) -> &V {
        &self.node.entry.value
    }
}
impl<K: Copy, V: Copy> ExecutionOrderedMapCell<K, V> {
    /// Explicit ordinary/allocation-policy route; never fabricates a request.
    pub fn prepare_in(
        key: K,
        value: V,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        policy.check_live()?;
        let node = node::prepared_copy(key, value, policy.into())?;
        policy.check_live()?;
        Ok(Self { node })
    }

    /// Prepare the actual two cells through the caller's original request.
    pub fn prepare_for_request(
        key: K,
        value: V,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        super::request::funded(request, |funding| {
            node::prepared_copy(key, value, funding).map(|node| Self { node })
        })
    }
}
impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for ExecutionOrderedMapCell<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionOrderedMapCell")
            .field("key", self.key())
            .field("value", self.value())
            .finish()
    }
}

impl<K: Ord + Copy, V: Copy> ExecutionOrderedMap<K, V> {
    /// Change only a Copy value, retaining its actual entry and reservation.
    /// Missing keys return None; shared path/entry storage refuses unchanged.
    /// No allocation, payload Clone or request admission occurs in this edit.
    pub fn set_existing_unique(
        &mut self,
        key: &K,
        value: V,
    ) -> Result<Option<V>, ExecutionOrderedMapUniqueDenial> {
        if !preflight::set(&mut self.root, key)? {
            return Ok(None);
        }
        Ok(Some(edit::set(&mut self.root, key, value)))
    }

    /// Insert a pre-admitted singleton without allocating an AVL path.
    /// Every refusal returns the SAME cell, and changes no map state. Copy
    /// updates use set_existing_unique; an occupied key never displaces storage.
    /// Arbitrary external Ord implementations are not promised allocation-free.
    pub fn insert_vacant_prepared(
        &mut self,
        cell: ExecutionOrderedMapCell<K, V>,
    ) -> Result<
        (),
        (
            ExecutionOrderedMapUniqueDenial,
            ExecutionOrderedMapCell<K, V>,
        ),
    > {
        if let Err(cause) = preflight::insert(&mut self.root, cell.key(), cell.node.bytes) {
            return Err((cause, cell));
        }
        edit::insert(&mut self.root, cell.node);
        Ok(())
    }

    /// Detach the actual removed singleton. Its data/grants remain live until
    /// the returned owner drops. All path, successor and rotation cells are
    /// checked for real uniqueness before any mutation; no fallback copies.
    /// Unaccounted nodes remain conservatively unaccounted after unique edits.
    pub fn remove_retaining_unique(
        &mut self,
        key: &K,
    ) -> Result<Option<ExecutionOrderedMapCell<K, V>>, ExecutionOrderedMapUniqueDenial> {
        if preflight::remove(&mut self.root, key)?.is_none() {
            return Ok(None);
        }
        Ok(Some(ExecutionOrderedMapCell {
            node: edit::remove(&mut self.root, key),
        }))
    }
}
