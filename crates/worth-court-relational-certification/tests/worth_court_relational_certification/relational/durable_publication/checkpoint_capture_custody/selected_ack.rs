//! Actual captured publication owners survive partial handoff and retirement.
//! Restored from 22e638d638:crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs.
//! Fixture adaptations: commit envelope cells are read through the owner's
//! `observe_commit_envelope_custody` census (selected directory and source
//! placement as weak allocation identities, so `same_owner` becomes
//! `same_allocation`); checkpoint selection through `observe_checkpoint_selection`;
//! the strict original frame read (no journal fallback, as the original's
//! `require_original_artifact(.., None)`) through
//! `read_original_commit_artifact_without_journal`; the
//! publication owner identity through `publication_subject().same_capture`;
//! the region count through `region_publication_count`; basis custody through
//! `RelationalBasisCustodyWitness`.
use super::*;
use worth_relational::facade::{
    durability::{
        RelationalCheckpointAcknowledgementDenial, RelationalCheckpointMember,
        RelationalCommitEnvelopeCustody,
    },
    history::CommitId,
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_publication::{create_entity, create_entity_outcome, durable_runtime},
};

fn work_limited_ack_request(work: u64) -> worth_execution::SerialRequest {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::new(1).unwrap(),
            Some(64 << 20),
            Some(work),
        ),
    );
    worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationSource::new().token(),
        None,
    )
}

fn cell(runtime: &RelationalRuntime, commit: CommitId) -> RelationalCommitEnvelopeCustody {
    runtime
        .observe_commit_envelope_custody()
        .into_iter()
        .find(|cell| cell.commit_id == commit)
        .expect("retained commit cell")
}

#[test]
fn selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    for name in ["selected-one", "selected-two", "selected-three"] {
        let result = create_entity_outcome(&execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &result);
    }
    let cells = runtime.observe_commit_envelope_custody();
    let entries = cells
        .iter()
        .map(|cell| cell.source_placement.clone().unwrap())
        .collect::<Vec<_>>();
    let checkpoint = owned_capture(&execution, &runtime, &medium, &[]);
    let original = checkpoint.publication_subject().unwrap().clone();
    let later = create_entity_outcome(&execution, &runtime, "after-actual-cut");
    let later_id = later.commit.commit_id;
    release_test_commit_snapshot(&runtime, &later);
    let later_entry = cell(&runtime, later_id).source_placement.unwrap();
    medium.file_checkpoint(checkpoint.clone());
    let limited = work_limited_ack_request(1);
    let request = ExecutionRequest::serial(&limited);
    let mut observed = None;
    let _outer = request.run_native_with_checkpoint(|context| {
        let result = checkpoint.acknowledge_selected(&request, context);
        observed = Some(matches!(
            result,
            Err(RelationalCheckpointAcknowledgementDenial::Stopped(_))
        ));
    });
    assert_eq!(observed, Some(true));
    assert!(cell(&runtime, cells[0].commit_id)
        .selected_directory
        .is_some());
    assert!(
        cells[1..]
            .iter()
            .all(|original| cell(&runtime, original.commit_id)
                .selected_directory
                .is_none()),
        "a real stop leaves every later covered cell untouched"
    );
    let later_cell = cell(&runtime, later_id);
    assert!(later_cell.selected_directory.is_none());
    assert!(later_cell
        .source_placement
        .unwrap()
        .same_allocation(&later_entry));
    assert!(
        runtime.observe_checkpoint_selection().selected.is_none(),
        "partial cell transfer is not directory placement or completion"
    );
    acknowledge(&execution, &checkpoint);
    assert!(original.same_capture(checkpoint.publication_subject().unwrap()));
    for (original, entry) in cells.iter().zip(&entries) {
        let now = cell(&runtime, original.commit_id);
        assert!(now.selected_directory.is_some());
        assert!(now.source_placement.unwrap().same_allocation(entry));
    }
    let later_cell = cell(&runtime, later_id);
    assert!(later_cell.selected_directory.is_none());
    assert!(later_cell
        .source_placement
        .unwrap()
        .same_allocation(&later_entry));
}

#[test]
fn selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(&execution, &runtime, "old-selected-root");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let retained_basis = basis.custody_witness();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    let first = owned_capture(&execution, &runtime, &medium, &[]);
    medium.file_checkpoint(first.clone());
    acknowledge(&execution, &first);
    let prior = runtime
        .observe_checkpoint_selection()
        .selected
        .unwrap()
        .directory;
    let current = first
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect::<Vec<_>>();
    drop(first);
    let old_frontier = runtime
        .observe_checkpoint_selection()
        .acknowledged_index_definition_frontier;
    let root = create_entity_outcome(&execution, &runtime, "new-selected-root");
    release_test_commit_snapshot(&runtime, &root);
    // This real definition has not appeared in any canonical publication yet.
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "pending-selected-definition".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap();
    // Fixture adaptation: publication maintenance no longer cold-reconstructs a
    // definition registered over existing records (bounded maintenance since
    // 2026-11-06), so the original's later publication first needs one real
    // build. A build publishes no canonical commit: the definition still has
    // not appeared in any canonical publication.
    let (_, built_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: root.commit.commit_id,
            branch_id: root.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
        &built_basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    drop(built_basis);
    let checkpoint = owned_capture(&execution, &runtime, &medium, &current);
    runtime.release_component_basis(lease).unwrap();
    runtime.run_branch_root_reclamation_pass();
    assert!(retained_basis.basis_retained());
    let original = checkpoint.publication_subject().unwrap().clone();
    medium.file_checkpoint(checkpoint.clone());
    let covered = checkpoint.covered_position().unwrap();
    let work = covered.0 + checkpoint.region_publication_count() as u64;
    let limited = work_limited_ack_request(work);
    let request = ExecutionRequest::serial(&limited);
    let mut refused = false;
    let _outer = request.run_native_with_checkpoint(|context| {
        refused = checkpoint.acknowledge_selected(&request, context).is_err();
    });
    assert!(refused);
    let selection = runtime.observe_checkpoint_selection();
    let selected = selection.selected.unwrap();
    assert_eq!(selected.covered_position, covered);
    assert!(selected.index_definition_frontier > old_frontier);
    assert_eq!(
        selection.acknowledged_index_definition_frontier, old_frontier,
        "DATA availability is not completed catalog acceptance"
    );
    assert!(
        prior.is_retained(),
        "displaced read is retained before retirement progress"
    );
    assert!(
        retained_basis.basis_retained(),
        "DATA pointer placement cannot release capture holds or complete ACK"
    );
    for cell in runtime.observe_commit_envelope_custody() {
        let directory = cell.selected_directory.unwrap();
        assert!(selected.directory.same_allocation(&directory));
        let frame = runtime
            .read_original_commit_artifact_without_journal(&execution, cell.commit_id)
            .unwrap()
            .unwrap();
        assert_eq!(frame.commit_id(), cell.commit_id);
    }
    let later = create_entity_outcome(&execution, &runtime, "publication-while-ack-pending");
    let canonical = runtime
        .replay_commit_envelope(&execution, later.commit.commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(canonical.index_definition_additions(), &[definition]);
    let later_entry = cell(&runtime, later.commit.commit_id)
        .source_placement
        .unwrap();
    release_test_commit_snapshot(&runtime, &later);
    acknowledge(&execution, &checkpoint);
    assert!(original.same_capture(checkpoint.publication_subject().unwrap()));
    let later_cell = cell(&runtime, later.commit.commit_id);
    assert!(later_cell.selected_directory.is_none());
    assert!(later_cell
        .source_placement
        .unwrap()
        .same_allocation(&later_entry));
    assert_eq!(
        runtime
            .observe_checkpoint_selection()
            .acknowledged_index_definition_frontier,
        selected.index_definition_frontier
    );
    assert!(!prior.is_retained());
    assert!(
        !retained_basis.basis_retained(),
        "only actual completed ACK releases original component holds"
    );
}
