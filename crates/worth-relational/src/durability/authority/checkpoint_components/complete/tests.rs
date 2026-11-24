//! Genuine producer components, original-request admission and actual holds.
use super::*;
use crate::branch::{
    basis_observation_inputs::BranchObservationInputs, basis_retention::retain_component_basis,
    reference::RelationalBranchReferenceCell, root::RelationalBranchRoot,
};
use crate::history::data::BranchId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::RelationalSchemaRegistry;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

// Borrow both requests explicitly. The metadata predecessor is admitted inside
// the SAME genuine epochs; finite component-only controls use a separate budget
// for that existing input, without claiming to price the entire Complete call.
fn capture_bound(
    history: &HistorySubsystem,
    metadata: &ExecutionRequest<'_, '_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedCompleteComponents, Stop> {
    let data = history
        .capture_checkpoint_branch_metadata(metadata, "complete component fixture")
        .map_err(|_| corrupt("genuine branch metadata fixture capture"))?;
    capture_complete(history, request, &data)
}
fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut charge = 0;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| charge = budget.charged_memory_bytes())
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    charge
}
#[test]
fn complete_component_capture_keeps_exact_external_cut_without_new_basis() {
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let request = ExecutionRequest::serial(&setup);
    let history = crate::branch::basis_admission_tests::installed_history(1);
    let schema = RelationalSchemaRegistry::new();
    let instrumentation = RuntimeInstrumentation::new();
    let inputs = BranchObservationInputs {
        history: &history,
        runtime_instance_id: history.runtime_instance_id,
        empty_schema: &schema,
        instrumentation: &instrumentation,
    };
    let main = &history.main_branch;
    let (descriptor_a, basis_a) = inputs
        .observe(&inputs.branch_identity(main).unwrap(), || None)
        .unwrap();
    let lease_a = retain_component_basis(
        &history,
        history.runtime_instance_id,
        &instrumentation,
        &basis_a,
    )
    .unwrap();
    // Keep A's observation so the final registration removal has its own counter window.
    let branch = BranchId("longer-branch".into());
    history.insert_branch_cell(
        RelationalBranchReferenceCell::empty(history.runtime_instance_id, branch.clone()).unwrap(),
    );
    history
        .install_branch_root(&branch, RelationalBranchRoot::empty())
        .unwrap();
    let (descriptor_b, basis_b) = inputs
        .observe(&inputs.branch_identity(&branch).unwrap(), || None)
        .unwrap();
    for branch in [main, &branch] {
        history
            .branch_cell(branch)
            .unwrap()
            .advance_metadata()
            .unwrap();
    }
    assert_ne!(
        history.branch_cell(main).unwrap().observation(),
        *descriptor_a.reference()
    );
    assert_ne!(
        history.branch_cell(&branch).unwrap().observation(),
        *descriptor_b.reference()
    );
    let before = instrumentation.basis_counters();
    let (entries, _, mutations) = history.basis_registry_metrics();
    crate::history::retention::assert_controlled_component_capture_winner(
        &history,
        &instrumentation,
        &basis_b,
        lease_a,
        || {
            let captured = capture_bound(&history, &request, &request).unwrap();
            let keys = captured
                .observations()
                .map(|(descriptor, _)| rmp_serde::to_vec_named(descriptor).unwrap())
                .collect::<Vec<_>>();
            assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
            captured
                .observations()
                .map(|(descriptor, _)| descriptor.clone())
                .collect()
        },
    );
    let after = instrumentation.basis_counters();
    assert_eq!(after.basis_observations, before.basis_observations);
    let (entries_after, _, mutations_after) = history.basis_registry_metrics();
    assert_eq!(mutations_after, mutations);
    assert_eq!(entries_after, entries);
    // Only this final original owner drop removes A's existing registration.
    drop(basis_a);
    let (released_entries, _, released_mutations) = history.basis_registry_metrics();
    assert_eq!(released_entries.checked_add(1), Some(entries_after));
    assert_eq!(released_mutations, mutations_after.checked_add(1).unwrap());
    drop(basis_b);
    assert!(!history.has_detached_retention_obligations());
}
#[test]
fn complete_component_capture_refuses_original_roster_before_effects() {
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let metadata = ExecutionRequest::serial(&setup);
    let history = crate::branch::basis_admission_tests::installed_history(1);
    let before = history.retention_cost_counters();
    let bill = std::alloc::Layout::array::<Current>(1).unwrap().size() as u64;
    let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let refusal = capture_bound(&history, &metadata, &ExecutionRequest::serial(&host)).unwrap_err();
    match refusal {
        Stop::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested,
                admitted,
                ancestor: 0,
            },
        ) => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("must enter the original component roster quote: {other:?}"),
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(history.retention_cost_counters(), before);
    assert!(!history.has_detached_retention_obligations());
}
#[test]
fn complete_component_holds_keep_actual_read_obligations_until_drop() {
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let metadata = ExecutionRequest::serial(&setup);
    let history = crate::branch::basis_admission_tests::installed_history(1);
    let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let (_, _, mutations) = history.basis_registry_metrics();
    let captured = capture_bound(&history, &metadata, &ExecutionRequest::serial(&host)).unwrap();
    assert_eq!(captured.observations().len(), 1);
    assert!(history.has_detached_retention_obligations());
    let data_and_holds = budget.charged_memory_bytes();
    let holds = captured.into_holds();
    assert!(history.has_detached_retention_obligations());
    assert!(budget.charged_memory_bytes() > 0);
    assert!(
        budget.charged_memory_bytes() < data_and_holds,
        "pure descriptor DATA was reclaimed before HOLDS"
    );
    assert_eq!(history.basis_registry_metrics().2, mutations);
    drop(holds);
    assert!(!history.has_detached_retention_obligations());
    assert_eq!(budget.charged_memory_bytes(), 0);
}
