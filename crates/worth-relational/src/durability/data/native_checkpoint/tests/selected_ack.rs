//! Actual captured publication owners survive partial handoff and retirement.
use super::*;

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
#[test]
fn selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture() {
    use crate::tests::support::*;
    use worth_execution::ExecutionRequest;
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    for name in ["selected-one", "selected-two", "selected-three"] {
        let result = create_entity_outcome(&execution, &runtime, name);
        release_test_commit_snapshot(&runtime, &result);
    }
    let cells = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| artifact.envelope_residency())
        .collect::<Vec<_>>();
    let entries = cells
        .iter()
        .map(|cell| cell.source_placement().unwrap().0)
        .collect::<Vec<_>>();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: runtime.durability.cold_source(),
            },
        )
        .unwrap();
    let original = checkpoint.publication.as_ref().unwrap().clone();
    let later = create_entity_outcome(&execution, &runtime, "after-actual-cut");
    let later_id = later.commit.commit_id;
    release_test_commit_snapshot(&runtime, &later);
    let later_cell = runtime
        .history
        .commit_artifact(later_id)
        .unwrap()
        .envelope_residency();
    let later_entry = later_cell.source_placement().unwrap().0;
    attached_test_medium(&runtime).file_checkpoint(checkpoint.clone());
    let limited = work_limited_ack_request(1);
    let request = ExecutionRequest::serial(&limited);
    let mut observed = None;
    let _outer = request.run_native_with_checkpoint(|context| {
        let result = checkpoint.acknowledge_selected(&request, context);
        observed = Some(matches!(
            result,
            Err(crate::durability::medium::RelationalCheckpointAcknowledgementDenial::Stopped(_))
        ));
    });
    assert_eq!(observed, Some(true));
    assert!(cells[0].selected_directory().is_some());
    assert!(
        cells[1..]
            .iter()
            .all(|cell| cell.selected_directory().is_none()),
        "a real stop leaves every later covered cell untouched"
    );
    assert!(later_cell.selected_directory().is_none());
    assert!(later_cell
        .source_placement()
        .unwrap()
        .0
        .same_owner(&later_entry));
    assert!(
        runtime
            .durability
            .checkpoint_selection()
            .snapshot()
            .is_none(),
        "partial cell transfer is not directory placement or completion"
    );
    execution
        .run_native_with_checkpoint(|context| checkpoint.acknowledge_selected(&execution, context))
        .unwrap()
        .0
        .unwrap();
    assert!(Arc::ptr_eq(
        &original,
        checkpoint.publication.as_ref().unwrap()
    ));
    for (cell, entry) in cells.iter().zip(&entries) {
        assert!(cell.selected_directory().is_some());
        assert!(cell.source_placement().unwrap().0.same_owner(entry));
    }
    assert!(later_cell.selected_directory().is_none());
    assert!(later_cell
        .source_placement()
        .unwrap()
        .0
        .same_owner(&later_entry));
}
#[test]
fn selected_ack_accepts_its_catalog_before_later_ordinary_publication() {
    use crate::tests::support::*;
    use worth_execution::ExecutionRequest;
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity(&execution, &runtime, "old-selected-root");
    let first = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: runtime.durability.cold_source(),
            },
        )
        .unwrap();
    attached_test_medium(&runtime).file_checkpoint(first.clone());
    execution
        .run_native_with_checkpoint(|context| first.acknowledge_selected(&execution, context))
        .unwrap()
        .0
        .unwrap();
    let current = first
        .members()
        .iter()
        .map(crate::durability::data::RelationalCheckpointMember::locator)
        .collect::<Vec<_>>();
    drop(first);
    let old_frontier = runtime
        .durability
        .checkpoint_selection()
        .acknowledged_index_definition_frontier();
    create_entity(&execution, &runtime, "new-selected-root");
    let definition = runtime
        .index_authority()
        .register(crate::indexes::data::DerivedIndexDefinition {
            index_id: crate::indexes::data::DerivedIndexId(0),
            name: "selected-definition".into(),
            branch_scoped: true,
            kind: crate::indexes::data::DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        &execution,
        crate::indexes::data::DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: basis.identity().branch_id().clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.basis_denial.is_none(), "{built:?}");
    assert!(built.execution_denial.is_none(), "{built:?}");
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert_eq!(built.generations.len(), 1);
    drop(basis);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current,
                source: runtime.durability.cold_source(),
            },
        )
        .unwrap();
    let original = checkpoint.publication.as_ref().unwrap().clone();
    let covered = checkpoint.covered_position().unwrap();
    attached_test_medium(&runtime).file_checkpoint(checkpoint.clone());
    execution
        .run_native_with_checkpoint(|context| checkpoint.acknowledge_selected(&execution, context))
        .unwrap()
        .0
        .unwrap();
    let selected = runtime
        .durability
        .checkpoint_selection()
        .snapshot()
        .unwrap();
    assert_eq!(selected.manifest.covered_position, covered);
    assert!(selected.manifest.index_definition_frontier > old_frontier);
    assert_eq!(
        runtime
            .durability
            .checkpoint_selection()
            .acknowledged_index_definition_frontier(),
        selected.manifest.index_definition_frontier
    );
    assert!(Arc::ptr_eq(
        &original,
        checkpoint.publication.as_ref().unwrap()
    ));
    for artifact in runtime.history.commit_artifacts() {
        let cell = artifact.envelope_residency();
        assert!(Arc::ptr_eq(&selected, &cell.selected_directory().unwrap()));
        let frame = cell.require_original_artifact(&execution, None).unwrap();
        assert_eq!(frame.commit_id(), artifact.commit_id());
    }
    let later = create_entity_outcome(&execution, &runtime, "publication-after-complete-ack");
    let canonical = runtime
        .history
        .recorded_commit_envelope(later.commit.commit_id)
        .unwrap();
    assert!(
        canonical.index_definition_delta.definitions().is_empty(),
        "the acknowledged definition is accepted; later publication does not reintroduce it"
    );
    let later_cell = runtime
        .history
        .commit_artifact(later.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let later_entry = later_cell.source_placement().unwrap().0;
    release_test_commit_snapshot(&runtime, &later);
    assert!(later_cell.selected_directory().is_none());
    assert!(later_cell
        .source_placement()
        .unwrap()
        .0
        .same_owner(&later_entry));
}
