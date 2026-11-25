//! The borrowed Complete view is read by the one format-3 reader, with its selections.
use super::super::super::funding::with_budget;
use super::*;
use contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use contracts::history::data::coordinates::{BranchId, CommitId};
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey;
use worth_execution::SerialMemoryBudget;
use worth_foundational::facade::AspectValue;
use worth_relational_contracts::internal_port as contracts;

fn generation(id: u64, changed_key: u64) -> DerivedIndexGeneration {
    let entries = (0..100)
        .map(|key| {
            let value = if key == changed_key { id } else { key };
            (
                AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::UInt64(key)),
                vec![EntityId::new(PartitionId::main(), value, 1)],
            )
        })
        .collect::<BTreeMap<_, _>>()
        .into();
    DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(id),
        index_id: DerivedIndexId(1),
        source_commit_id: CommitId(id),
        source_branch_id: BranchId("main".into()),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId("main".into()),
            version_id: VersionId(id),
            schema_version: SchemaVersionId(1),
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(entries),
    }
}
fn basis(commit: u64, generation: u64) -> IndexBasisCheckpoint {
    IndexBasisCheckpoint {
        branch: BranchId("main".into()),
        commit: CommitId(commit),
        version: VersionId(commit),
        schema: SchemaVersionId(1),
        empty: vec![DerivedIndexId(2)],
        bindings: vec![(DerivedIndexId(1), DerivedIndexGenerationId(generation))],
    }
}

#[test]
fn complete_view_roundtrips_same_index_generations_and_only_retained_selections() {
    worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        // Two generations of one index: the owned writer would delta the second.
        let generations = [Arc::new(generation(1, 90)), Arc::new(generation(2, 90))];
        // Commit 7 carries generation 2, authored at version 2, to a later root.
        let bases = [basis(5, 1), basis(7, 2)];
        let retained = |basis: &IndexBasisCheckpoint| basis.commit == CommitId(7);
        let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
        let view = with_budget(&budget, |work| {
            CompleteIndexCheckpointView::capture_funded(&generations, &bases, &retained, work)
        })
        .unwrap();
        assert!(budget.charged_memory_bytes() > 0);
        let wire = rmp_serde::to_vec_named(&view).unwrap();
        drop(view);
        assert_eq!(budget.charged_memory_bytes(), 0);
        // The funded reader's quote walks the same payload, selections included.
        assert!(
            with_budget(&budget, |work| quote_decode(BorrowedPayload(&wire), work)).unwrap() > 0
        );
        let decoded: DerivedIndexCheckpointArtifacts = rmp_serde::from_slice(&wire).unwrap();
        assert!(decoded
            .generations
            .iter()
            .all(|row| row.base_generation_id.is_none()));
        assert_eq!(
            decoded.readmit(fixture_lane).unwrap(),
            generations
                .iter()
                .map(|generation| generation.as_ref().clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(decoded.bases(), &bases[1..]);
    })
}

#[test]
fn complete_view_capture_stops_at_its_funding_budget() {
    let generations = [Arc::new(generation(1, 90))];
    let retained = |_: &IndexBasisCheckpoint| true;
    let budget = SerialMemoryBudget::new(Some(64));
    assert!(matches!(
        with_budget(&budget, |work| CompleteIndexCheckpointView::capture_funded(
            &generations,
            &[],
            &retained,
            work
        )),
        Err(Stop::Request(_) | Stop::Allocation(_))
    ));
}
