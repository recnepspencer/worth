//! Real wire tuples retain funded generation DATA and refuse original stops.
use super::*;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::indexes::data::*;
use crate::schema::data::SchemaVersionId;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use std::collections::BTreeMap;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
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
fn original_full_and_delta_index_tuples_preserve_all_entry_families_and_funding() {
    let mut expected = generations();
    let mut repeated = expected[0].clone();
    repeated.generation_id = DerivedIndexGenerationId(5);
    repeated.source_commit_id = CommitId(5);
    repeated.applicability.version_id = VersionId(5);
    expected.push(repeated);
    expected.sort_by_key(|generation| (generation.index_id, generation.generation_id));
    let full = DerivedIndexArtifacts::new(expected.clone());
    let delta =
        DerivedIndexCheckpointArtifacts::capture(expected.iter().cloned().map(Arc::new).collect())
            .unwrap();
    assert_eq!(
        serde_json::to_value(&delta).unwrap()["generations"][1]["base_generation_id"],
        1
    );
    let payloads = [
        rmp_serde::to_vec_named(&(&full, None::<DerivedIndexCheckpointArtifacts>, 0_u16)).unwrap(),
        rmp_serde::to_vec_named(&(DerivedIndexArtifacts::default(), Some(delta), 2_u16)).unwrap(),
    ];
    for payload in payloads {
        let budget = SerialMemoryBudget::new(Some(8 << 20));
        let host =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let baseline = budget.charged_memory_bytes();
        let captured = decode(&payload, &ExecutionRequest::serial(&host)).unwrap();
        assert_eq!(
            captured
                .generations
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        assert!(budget.charged_memory_bytes() > baseline);
        drop(captured);
        assert_eq!(budget.charged_memory_bytes(), baseline);
    }
}
#[test]
fn original_index_decode_refuses_cancelled_and_exhausted_invoking_requests_without_retaining_data()
{
    let full = DerivedIndexArtifacts::new(generations());
    let payload =
        rmp_serde::to_vec_named(&(full, None::<DerivedIndexCheckpointArtifacts>, 0_u16)).unwrap();
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        decode(&payload, &ExecutionRequest::serial(&host)),
        Err(Stop::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    let budget = SerialMemoryBudget::new(Some(1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    assert!(matches!(
        decode(&payload, &ExecutionRequest::serial(&host)),
        Err(Stop::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
