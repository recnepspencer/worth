//! Value-only full and delta tuples preserve every index entry family.
use super::*;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::indexes::data::*;
use crate::schema::data::SchemaVersionId;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;

fn generations() -> Vec<DerivedIndexGeneration> {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let other = EntityId::new(PartitionId::main(), 2, 1);
    let relation = RelationId::new(PartitionId::main(), 3, 1);
    let key = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::UInt64(3));
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
        DerivedIndexEntries::RelationJoin(
            BTreeMap::from([(
                RelationJoinKey::new(entity, other),
                vec![RelationJoinEntry::new(entity, relation, relation)],
            )])
            .into(),
        ),
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
