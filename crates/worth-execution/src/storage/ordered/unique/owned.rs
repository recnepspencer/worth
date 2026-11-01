//! Move native payload custody through the same preflighted actual cell owners.
use super::*;

impl<K: Ord + Copy, V> ExecutionOrderedMapCell<K, V> {
    /// Move caller-owned native payload into admitted framework cells. Nested
    /// payload storage retains its own ownership and admission contract.
    pub fn prepare_owned_native_for_request(
        key: K,
        value: V,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        crate::storage::ordered::request::funded(request, |funding| {
            node::prepared_owned(key, value, funding).map(|node| Self { node })
        })
    }

    /// Explicit allocation-policy route; this does not invent a request or
    /// retrospectively fund the caller's native payload.
    pub fn prepare_owned_native_in(
        key: K,
        value: V,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        policy.check_live()?;
        let node = node::prepared_owned(key, value, policy.into())?;
        policy.check_live()?;
        Ok(Self { node })
    }
}

impl<K: Ord + Copy, V> ExecutionOrderedMap<K, V> {
    /// Move a prepared native singleton without allocating or cloning payload.
    /// A refusal returns the same cell and leaves the original map unchanged.
    pub fn insert_owned_vacant_prepared(
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

    /// Retain the same removed node, entry and original grants without cloning
    /// or exposing mutable payload. Shared storage refuses before any edit.
    pub fn remove_owned_retaining_unique(
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

#[cfg(test)]
mod tests;
