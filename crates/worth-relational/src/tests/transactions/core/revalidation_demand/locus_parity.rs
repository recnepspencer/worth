//! Staging admission and overlay indexing consume one locus classification for
//! every entity intent variant.

use std::collections::BTreeSet;

use super::fixtures::*;
use crate::tests::support::*;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, AspectFieldPatch, DeleteEntityIntent, EntitySpec,
    ReplaceEntityIntent, RevalidateEntityIntent, UpdateEntityFieldsIntent,
};
use worth_foundational::facade::PortableRecordAspectPatch;

#[test]
fn every_entity_intent_records_exactly_its_authoritative_locus() {
    let runtime = strictness_runtime();
    let update = create_entity(&runtime, COMPLIANT_NAME);
    let patch = create_entity(&runtime, COMPLIANT_NAME);
    let replace = create_entity(&runtime, COMPLIANT_NAME);
    let delete = create_entity(&runtime, COMPLIANT_NAME);
    let revalidate = create_entity(&runtime, COMPLIANT_NAME);

    let batch = WorkerIntentBatch::new("all-entity-intent-loci")
        .push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: update,
                fields: AspectFieldPatch::default(),
            },
        )))
        .push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: patch,
                aspect_patch: PortableRecordAspectPatch::new([]),
            }),
        ))
        .push(MutationIntent::Entity(EntityMutationIntent::Replace(
            ReplaceEntityIntent {
                entity_id: replace,
                replacement: EntitySpec {
                    partition_id: replace.partition_id,
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: AspectFieldPatch::default(),
                },
            },
        )))
        .push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: delete },
        )))
        .push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
            RevalidateEntityIntent {
                entity_id: revalidate,
            },
        )));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch)
        .expect("all five loci fit the default footprint ceiling");

    let write_entities = transaction
        .footprint()
        .writes()
        .filter_map(|locus| match locus {
            crate::facade::mvcc::RelationalTransactionWriteLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity_id),
            ) => Some(*entity_id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let read_entities = transaction
        .footprint()
        .reads()
        .filter_map(|locus| match locus {
            crate::facade::mvcc::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity_id),
            ) => Some(*entity_id),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(transaction.footprint().writes().len(), 4);
    assert_eq!(transaction.footprint().reads().len(), 1);
    assert_eq!(
        write_entities,
        BTreeSet::from([update, patch, replace, delete])
    );
    assert_eq!(read_entities, BTreeSet::from([revalidate]));
}
