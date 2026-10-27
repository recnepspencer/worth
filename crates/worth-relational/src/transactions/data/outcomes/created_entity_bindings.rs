use std::collections::BTreeMap;

use crate::identity::data::EntityId;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::StringInterner;
use crate::transactions::data::CreatedEntityRef;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StableCreatedEntityRef {
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: String,
}

/// Owner-minted correspondence between a committed create intent and its record identity.
///
/// The map is assembled while Relational applies the authoritative mutation
/// and remains a private field of that commit's result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommitCreatedEntityBindings {
    by_exact_reference: BTreeMap<CreatedEntityRef, EntityId>,
    by_stable_reference: BTreeMap<StableCreatedEntityRef, EntityId>,
}

impl CommitCreatedEntityBindings {
    pub(crate) fn from_owner_map(
        by_create_reference: BTreeMap<CreatedEntityRef, EntityId>,
        symbols: &StringInterner,
    ) -> Self {
        let by_stable_reference = by_create_reference
            .iter()
            .filter_map(|(created, entity_id)| {
                let client_key = created
                    .client_key
                    .resolve_with_interner(symbols)?
                    .to_owned();
                Some((
                    StableCreatedEntityRef {
                        partition_id: created.partition_id,
                        kind_id: created.kind_id,
                        client_key,
                    },
                    *entity_id,
                ))
            })
            .collect();
        Self {
            by_exact_reference: by_create_reference,
            by_stable_reference,
        }
    }

    /// Bound both possible tree searches and the stable fallback's key copy.
    /// A node compares at most eleven keys; binary height bounds are looser
    /// than B-tree height. Each comparison reaches two numeric identities,
    /// the key discriminator and its length/symbol, plus actual raw bytes.
    pub(crate) fn resolve_work_bound(&self, created: &CreatedEntityRef) -> Option<u64> {
        let raw_bytes = created.client_key.as_raw_str().map(str::len);
        let comparison_width = raw_bytes.unwrap_or(0).checked_add(4)?;
        let tree_work = |entries: usize| {
            let levels = usize::BITS as usize - entries.max(1).leading_zeros() as usize;
            entries
                .min(11)
                .checked_mul(levels)?
                .checked_add(1)?
                .checked_mul(comparison_width)
        };
        let mut work = tree_work(self.by_exact_reference.len())?;
        if let Some(bytes) = raw_bytes {
            work = work
                .checked_add(bytes)?
                .checked_add(tree_work(self.by_stable_reference.len())?)?;
        }
        u64::try_from(work).ok()
    }

    /// Recover diagnostic creation meaning from this apply's actual correspondence.
    /// Only the refusal path walks these retained bindings; ordinary successful
    /// validation allocates no reverse index. The caller checkpoints each entry.
    pub(crate) fn original_reference(
        &self,
        entity_id: EntityId,
        mut checkpoint: impl FnMut() -> bool,
    ) -> Option<CreatedEntityRef> {
        for (stable, allocated) in &self.by_stable_reference {
            if !checkpoint() {
                return None;
            }
            if *allocated == entity_id {
                return Some(CreatedEntityRef {
                    partition_id: stable.partition_id,
                    kind_id: stable.kind_id,
                    client_key: crate::symbols::data::ClientKey::raw(stable.client_key.clone()),
                });
            }
        }
        for (created, allocated) in &self.by_exact_reference {
            if !checkpoint() {
                return None;
            }
            if *allocated == entity_id {
                return Some(created.clone());
            }
        }
        None
    }

    pub(crate) fn resolve(&self, created: &CreatedEntityRef) -> Option<EntityId> {
        if let Some(entity_id) = self.by_exact_reference.get(created) {
            return Some(*entity_id);
        }
        let client_key = created.client_key.as_raw_str()?;
        self.by_stable_reference
            .get(&StableCreatedEntityRef {
                partition_id: created.partition_id,
                kind_id: created.kind_id,
                client_key: client_key.to_owned(),
            })
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::data::ClientKey;

    #[test]
    fn every_create_reference_resolves_its_own_distinct_owner_identity() {
        let partition = PartitionId::main();
        let kind = KindId::new(17);
        let first = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: ClientKey::raw("first-created-record"),
        };
        let second = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: ClientKey::raw("second-created-record"),
        };
        let first_id = EntityId::new(partition, 41, 1);
        let second_id = EntityId::new(partition, 42, 1);
        let bindings = CommitCreatedEntityBindings::from_owner_map(
            BTreeMap::from([(first.clone(), first_id), (second.clone(), second_id)]),
            &StringInterner::default(),
        );

        assert_eq!(bindings.resolve(&first), Some(first_id));
        assert_eq!(bindings.resolve(&second), Some(second_id));
        assert_ne!(bindings.resolve(&first), bindings.resolve(&second));
        assert_eq!(bindings.original_reference(first_id, || true), Some(first));
        assert_eq!(
            bindings.original_reference(second_id, || true),
            Some(second)
        );
        assert!(bindings
            .original_reference(EntityId::new(partition, 99, 1), || true)
            .is_none());
    }

    #[test]
    fn symbol_backed_create_reference_resolves_without_raw_reconstruction() {
        let partition = PartitionId::main();
        let kind = KindId::new(18);
        let mut symbols = StringInterner::default();
        let symbol = symbols.intern("symbol-created-record");
        let created = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: ClientKey::symbol(symbol),
        };
        let entity_id = EntityId::new(partition, 43, 1);
        let bindings = CommitCreatedEntityBindings::from_owner_map(
            BTreeMap::from([(created.clone(), entity_id)]),
            &symbols,
        );

        assert_eq!(bindings.resolve(&created), Some(entity_id));
        drop(symbols);
        let diagnostic = bindings.original_reference(entity_id, || true).unwrap();
        assert_eq!(diagnostic.partition_id, partition);
        assert_eq!(diagnostic.kind_id, kind);
        assert_eq!(
            diagnostic.client_key.as_raw_str(),
            Some("symbol-created-record")
        );
        let mut checkpoint_reached = false;
        assert!(bindings
            .original_reference(entity_id, || {
                checkpoint_reached = true;
                false
            })
            .is_none());
        assert!(checkpoint_reached);
    }

    #[test]
    fn foreign_symbol_identity_keeps_exact_owner_mapping_without_inventing_raw_alias() {
        let partition = PartitionId::main();
        let kind = KindId::new(19);
        let mut source_symbols = StringInterner::default();
        let symbol = source_symbols.intern("foreign-symbol-created-record");
        let created = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: ClientKey::symbol(symbol),
        };
        let entity_id = EntityId::new(partition, 44, 1);
        let bindings = CommitCreatedEntityBindings::from_owner_map(
            BTreeMap::from([(created.clone(), entity_id)]),
            &StringInterner::default(),
        );

        assert_eq!(bindings.resolve(&created), Some(entity_id));
        assert_eq!(
            bindings.resolve(&CreatedEntityRef {
                partition_id: partition,
                kind_id: kind,
                client_key: ClientKey::raw("foreign-symbol-created-record"),
            }),
            None
        );
    }
}
