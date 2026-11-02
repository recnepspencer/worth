//! Exact selected index refresh admits Head without fetching omitted history.
use super::cold_index_fixture::reopened_historical;
use super::cold_source::ColdRead;
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexEntries, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenialKind,
};

#[test]
fn cold_selected_index_refresh_preserves_budget_gate_and_reads_head_without_history() {
    let serial = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&serial);
    let fixture = reopened_historical(execution);
    let runtime = &fixture.runtime;
    let (descriptor, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let commit = basis.observation().commit_id().unwrap();
    assert_ne!(commit, fixture.generation.source_commit_id);
    let selected = fixture
        .manifest
        .roots
        .iter()
        .find(|root| root.commit_id == commit)
        .unwrap();
    let region = selected
        .regions
        .iter()
        .find(|region| region.partition_id == fixture.entity.partition_id)
        .unwrap();
    let head = region.head;
    let history = region.history;
    fixture
        .source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(history), Cause::MissingMember);
    assert!(
        basis
            .inner
            .root
            .partition_state(region.partition_id)
            .is_err(),
        "Full starts cold"
    );
    let position = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(runtime).offered().len();
    let request = DerivedIndexBuildRequest {
        source_commit_id: commit,
        branch_id: descriptor.branch_id().clone(),
        index_ids: vec![fixture.index],
    };
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    };
    let denied = runtime
        .index_authority()
        .refresh_for_basis(request.clone(), &basis, None, budget, execution)
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    assert!(!fixture.source.member_reads.lock().unwrap().contains(&head));
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&history));
    let result = runtime
        .index_authority()
        .refresh_for_basis(
            request,
            &basis,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_cold_record_slots: None,
                ..budget
            },
            execution,
        )
        .unwrap();
    let generation = result
        .generations
        .iter()
        .find(|generation| generation.index_id == fixture.index)
        .unwrap();
    assert_eq!(generation.source_commit_id, commit);
    assert_ne!(
        generation.entries, fixture.generation.entries,
        "current projection uses the changed field"
    );
    let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
        panic!("the declared entity field index keeps its family");
    };
    assert!(entries
        .values()
        .any(|owners| owners.contains(&fixture.entity)));
    assert!(fixture.source.member_reads.lock().unwrap().contains(&head));
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&history));
    assert!(
        basis
            .inner
            .root
            .partition_state(region.partition_id)
            .is_err(),
        "refresh never admits Full"
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(attached_test_medium(runtime).offered().len(), offered);
}
