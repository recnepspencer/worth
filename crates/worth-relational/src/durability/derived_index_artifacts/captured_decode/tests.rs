//! Value-only full and delta tuples preserve every index entry family.
use super::*;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::indexes::data::*;
use crate::schema::data::SchemaVersionId;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AspectValue;

fn generations() -> Vec<DerivedIndexGeneration> {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let other = EntityId::new(PartitionId::main(), 2, 1);
    let relation = RelationId::new(PartitionId::main(), 3, 1);
    let key = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::UInt64(3));
    // A maintained join index carries its leg state, as every real producer writes it.
    let mut join: RelationJoinIndex = BTreeMap::from([(
        RelationJoinKey::new(entity, other),
        vec![RelationJoinEntry::new(entity, relation, relation)],
    )])
    .into();
    let leg = || {
        BTreeMap::from([(
            entity,
            BTreeMap::from([(other, BTreeSet::from([relation]))]),
        )])
    };
    join.install_legs(JoinLegsState::cold(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        leg(),
        leg(),
    ));
    let entries = [
        DerivedIndexEntries::EntityField(BTreeMap::from([(key.clone(), vec![entity])]).into()),
        DerivedIndexEntries::RelationField(BTreeMap::from([(key, vec![relation])]).into()),
        DerivedIndexEntries::RelatedEntityOrdering(
            BTreeMap::from([(
                entity,
                vec![RelatedEntityOrderingEntry::new(
                    vec![AspectValue::UInt64(7)],
                    other,
                    relation,
                )],
            )])
            .into(),
        ),
        DerivedIndexEntries::RelationJoin(join),
    ];
    entries
        .into_iter()
        .enumerate()
        .map(|(ordinal, entries)| {
            let id = ordinal as u64 + 1;
            DerivedIndexGeneration {
                generation_id: DerivedIndexGenerationId(id),
                index_id: DerivedIndexId(id),
                source_commit_id: CommitId(id),
                source_branch_id: BranchId("original-branch".into()),
                applicability: DerivedIndexApplicability {
                    branch_id: BranchId("original-branch".into()),
                    version_id: VersionId(id),
                    schema_version: SchemaVersionId(1),
                },
                status: DerivedIndexPublicationStatus::Published,
                entries,
            }
        })
        .collect()
}
#[test]
fn full_and_delta_index_tuples_preserve_all_entry_families() {
    let mut expected = generations();
    let mut repeated = expected[0].clone();
    repeated.generation_id = DerivedIndexGenerationId(5);
    repeated.source_commit_id = CommitId(5);
    repeated.applicability.version_id = VersionId(5);
    expected.push(repeated);
    expected.sort_by_key(|generation| (generation.index_id, generation.generation_id));
    let full = DerivedIndexArtifacts::new(expected.clone());
    let delta = DerivedIndexCheckpointArtifacts::capture(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        expected.iter().cloned().map(Arc::new).collect(),
        Vec::new(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&delta).unwrap()["generations"][1]["base_generation_id"],
        1
    );
    let payloads = [
        rmp_serde::to_vec_named(&(&full, None::<DerivedIndexCheckpointArtifacts>, 0_u16)).unwrap(),
        rmp_serde::to_vec_named(&(
            DerivedIndexArtifacts::default(),
            Some(delta),
            DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        ))
        .unwrap(),
    ];
    for payload in payloads {
        let (full, delta, format): (
            DerivedIndexArtifacts,
            Option<DerivedIndexCheckpointArtifacts>,
            u16,
        ) = rmp_serde::from_slice(&payload).unwrap();
        assert!(DerivedIndexCheckpointArtifacts::supports_outer_format(
            format,
            delta.is_some(),
        ));
        let generations = match delta {
            Some(delta) => {
                assert!(full.is_empty());
                delta
                    .readmit(&crate::storage::substrate::maintenance::ReconstructionLane::oracle())
                    .unwrap()
            }
            None => full.into_generations(),
        };
        assert_eq!(generations, expected);
    }
}

#[test]
fn funded_decode_admits_every_entry_family_in_both_tuples() {
    // One generation per index, so the delta tuple carries full rows only.
    let expected = generations();
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let delta = DerivedIndexCheckpointArtifacts::capture(
        &lane,
        expected.iter().cloned().map(Arc::new).collect(),
        Vec::new(),
    )
    .unwrap();
    let payloads = [
        rmp_serde::to_vec_named(&(
            DerivedIndexArtifacts::new(expected.clone()),
            None::<DerivedIndexCheckpointArtifacts>,
            0_u16,
        ))
        .unwrap(),
        rmp_serde::to_vec_named(&(
            DerivedIndexArtifacts::default(),
            Some(delta),
            DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        ))
        .unwrap(),
    ];
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    for payload in payloads {
        let decoded = decode(&lane, &payload, &request).unwrap();
        assert_eq!(
            decoded
                .generations
                .iter()
                .map(|generation| generation.as_ref().clone())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn join_leg_state_is_priced_per_decoded_container() {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let relations: Vec<_> = (0..40)
        .map(|slot| RelationId::new(PartitionId::main(), 10 + slot, 1))
        .collect();
    // Each relation is its own (shared, external) pair: the sparse worst case.
    let leg = || {
        relations
            .iter()
            .enumerate()
            .map(|(ordinal, relation)| {
                (
                    EntityId::new(PartitionId::main(), 100 + ordinal as u64, 1),
                    BTreeMap::from([(entity, BTreeSet::from([*relation]))]),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    let bare = RelationJoinIndex::default();
    let mut maintained = bare.clone();
    maintained.install_legs(JoinLegsState::cold(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        leg(),
        leg(),
    ));
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let quote = |index: RelationJoinIndex| {
        let wire = rmp_serde::to_vec_named(&DerivedIndexEntries::RelationJoin(index)).unwrap();
        super::super::generation_decode_quote::entries_quote(
            crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(&wire),
            &request,
        )
        .unwrap()
    };
    // Per side: 40 outer entries, 40 inner maps and 40 relation sets, each at
    // least one im node of 64 relation-sized slots, plus the inverse map.
    let floor = 2 * 80 * 64 * std::mem::size_of::<RelationId>() as u64;
    assert!(quote(maintained) >= quote(bare) + floor);
}
