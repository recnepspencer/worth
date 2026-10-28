use std::collections::BTreeMap;

use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::StringInterner;
use crate::transactions::data::{CreatedRelationRef, EntityReference};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StableEntityReference {
    Existing(EntityId),
    Created {
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StableCreatedRelationRef {
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: String,
    source: StableEntityReference,
    target: StableEntityReference,
}

/// Owner-minted correspondence between a relation create intent and its
/// allocated relation identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommitCreatedRelationBindings {
    by_exact_reference: BTreeMap<CreatedRelationRef, RelationId>,
    by_stable_reference: BTreeMap<StableCreatedRelationRef, RelationId>,
}

impl CommitCreatedRelationBindings {
    /// Adopt correspondence minted by the same commit's private consequence apply.
    pub(crate) fn append_owned(
        &mut self,
        mut generated: Self,
    ) -> Result<(), crate::transactions::data::CommitConflict> {
        if generated
            .by_exact_reference
            .keys()
            .any(|key| self.by_exact_reference.contains_key(key))
            || generated
                .by_stable_reference
                .keys()
                .any(|key| self.by_stable_reference.contains_key(key))
        {
            return Err(crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::KindSchemaMismatch {
                    detail:
                        "Native consequence create correspondence collides with an earlier apply"
                            .into(),
                },
            ));
        }
        self.by_exact_reference
            .append(&mut generated.by_exact_reference);
        self.by_stable_reference
            .append(&mut generated.by_stable_reference);
        Ok(())
    }

    /// Borrow the correspondence minted by native allocation, without a
    /// caller-selected lookup or reconstruction from client keys.
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = (&CreatedRelationRef, &RelationId)> {
        self.by_exact_reference.iter()
    }

    pub(crate) fn from_owner_map(
        by_create_reference: BTreeMap<CreatedRelationRef, RelationId>,
        symbols: &StringInterner,
    ) -> Self {
        let by_stable_reference = by_create_reference
            .iter()
            .filter_map(|(created, relation_id)| {
                Some((stable_reference(created, symbols)?, *relation_id))
            })
            .collect();
        Self {
            by_exact_reference: by_create_reference,
            by_stable_reference,
        }
    }

    pub(super) fn resolve(&self, created: &CreatedRelationRef) -> Option<RelationId> {
        if let Some(relation_id) = self.by_exact_reference.get(created) {
            return Some(*relation_id);
        }
        self.by_stable_reference
            .get(&stable_reference_without_interner(created)?)
            .copied()
    }
}

fn stable_reference(
    created: &CreatedRelationRef,
    symbols: &StringInterner,
) -> Option<StableCreatedRelationRef> {
    Some(StableCreatedRelationRef {
        partition_id: created.partition_id,
        kind_id: created.kind_id,
        client_key: created
            .client_key
            .resolve_with_interner(symbols)?
            .to_owned(),
        source: stable_entity_reference(&created.source, symbols)?,
        target: stable_entity_reference(&created.target, symbols)?,
    })
}

fn stable_reference_without_interner(
    created: &CreatedRelationRef,
) -> Option<StableCreatedRelationRef> {
    Some(StableCreatedRelationRef {
        partition_id: created.partition_id,
        kind_id: created.kind_id,
        client_key: created.client_key.as_raw_str()?.to_owned(),
        source: stable_entity_reference_without_interner(&created.source)?,
        target: stable_entity_reference_without_interner(&created.target)?,
    })
}

fn stable_entity_reference(
    reference: &EntityReference,
    symbols: &StringInterner,
) -> Option<StableEntityReference> {
    match reference {
        EntityReference::Existing(entity_id) => Some(StableEntityReference::Existing(*entity_id)),
        EntityReference::Created(created) => Some(StableEntityReference::Created {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created
                .client_key
                .resolve_with_interner(symbols)?
                .to_owned(),
        }),
    }
}

fn stable_entity_reference_without_interner(
    reference: &EntityReference,
) -> Option<StableEntityReference> {
    match reference {
        EntityReference::Existing(entity_id) => Some(StableEntityReference::Existing(*entity_id)),
        EntityReference::Created(created) => Some(StableEntityReference::Created {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: created.client_key.as_raw_str()?.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::KindId;
    use crate::symbols::data::ClientKey;

    #[test]
    fn exact_relation_intent_resolves_only_its_owner_identity() {
        let source = EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 0));
        let target = EntityReference::Existing(EntityId::new(PartitionId::main(), 2, 0));
        let created = CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(9),
            client_key: ClientKey::raw("route-a"),
            source: source.clone(),
            target: target.clone(),
        };
        let relation_id = RelationId::new(PartitionId::main(), 7, 1);
        let bindings = CommitCreatedRelationBindings::from_owner_map(
            BTreeMap::from([(created.clone(), relation_id)]),
            &StringInterner::default(),
        );

        assert_eq!(bindings.resolve(&created), Some(relation_id));
        assert_eq!(
            bindings.resolve(&CreatedRelationRef {
                client_key: ClientKey::raw("route-b"),
                ..created
            }),
            None
        );
    }
}
