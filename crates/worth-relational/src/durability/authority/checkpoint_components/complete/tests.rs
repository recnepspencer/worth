//! Genuine producer components, original-request admission and actual holds.
use super::*;
use crate::tests::support::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

// Borrow both requests explicitly. The metadata predecessor is admitted inside
// the SAME genuine epochs; finite component-only controls use a separate budget
// for that existing input, without claiming to price the entire Complete call.
fn capture_bound(
    runtime: &RelationalRuntime,
    metadata: &ExecutionRequest<'_, '_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedCompleteComponents, Stop> {
    let binding = runtime.configuration_binding();
    let config = binding.operation();
    let routes = runtime.history.canonical_checkpoint_gate();
    routes
        .with_checkpoint_selection(|_| {
            let data = runtime
                .history
                .capture_checkpoint_branch_metadata(metadata, &config.config.execution.runtime_name)
                .map_err(|_| corrupt("genuine branch metadata fixture capture"))?;
            capture_complete(runtime, request, &data)
        })
        .unwrap_or_else(|_| panic!("genuine component checkpoint gate must be settled"))
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
    let setup = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    create_entity(&request, &runtime, "original-a");
    let (_, basis_a) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let lease_a = runtime.retain_component_basis(&basis_a).unwrap();
    // Keep this original observation alive so concurrent lease release cannot
    // mix final registry removal into the capture-only counter window.
    let branch = create_branch_from_main(&runtime, "longer-branch");
    let (_, basis_b) = runtime
        .observe_branch(&runtime.branch_identity(&branch).unwrap())
        .unwrap();
    create_entity(&request, &runtime, "a-advanced");
    create_entity_in_partition_on_branch(
        &request,
        &runtime,
        "b-advanced",
        crate::identity::data::PartitionId::main(),
        branch.clone(),
    );
    let before = runtime.branch_basis_cost_counters();
    crate::history::retention::assert_controlled_component_capture_winner(
        &runtime,
        &basis_b,
        lease_a,
        || {
            let captured = capture_bound(&runtime, &request, &request).unwrap();
            let keys = captured
                .observations()
                .map(|(descriptor, _)| rmp_serde::to_vec_named(descriptor).unwrap())
                .collect::<Vec<_>>();
            assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
            let projection = captured
                .observations()
                .filter_map(|(descriptor, _)| {
                    descriptor.reference().target().as_basis().map(|target| {
                        (
                            descriptor.branch_id().clone(),
                            CommitId(target.selected_commit_id()),
                        )
                    })
                })
                .collect();
            drop(captured);
            projection
        },
    );
    let after = runtime.branch_basis_cost_counters();
    assert_eq!(after.basis_observations, before.basis_observations);
    assert_eq!(
        after.retained_basis_registry_mutations,
        before.retained_basis_registry_mutations
    );
    assert_eq!(
        after.retained_basis_registry_entries,
        before.retained_basis_registry_entries
    );
    // Only this final original owner drop removes A's existing registration.
    drop(basis_a);
    let released = runtime.branch_basis_cost_counters();
    assert_eq!(
        released.retained_basis_registry_entries.checked_add(1),
        Some(after.retained_basis_registry_entries)
    );
    assert_eq!(
        released.retained_basis_registry_mutations,
        after
            .retained_basis_registry_mutations
            .checked_add(1)
            .unwrap()
    );
    drop(basis_b);
    runtime.prepare_close().unwrap().finish();
}
#[test]
fn complete_component_capture_refuses_original_roster_before_effects() {
    let setup = crate::test_execution_policy::serial_request();
    let metadata = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let before = runtime.history.retention_cost_counters();
    let bill = std::alloc::Layout::array::<Current>(1).unwrap().size() as u64;
    let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let refusal = capture_bound(&runtime, &metadata, &ExecutionRequest::serial(&host)).unwrap_err();
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
    assert_eq!(runtime.history.retention_cost_counters(), before);
    runtime.prepare_close().unwrap().finish();
}
#[test]
fn complete_component_holds_keep_actual_read_obligations_until_drop() {
    let setup = crate::test_execution_policy::serial_request();
    let metadata = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let before = runtime.branch_basis_cost_counters();
    let captured = capture_bound(&runtime, &metadata, &ExecutionRequest::serial(&host)).unwrap();
    assert_eq!(captured.observations().len(), 1);
    assert!(runtime.history.has_detached_retention_obligations());
    let data_and_holds = budget.charged_memory_bytes();
    let holds = captured.into_holds();
    assert!(runtime.history.has_detached_retention_obligations());
    assert!(budget.charged_memory_bytes() > 0);
    assert!(
        budget.charged_memory_bytes() < data_and_holds,
        "pure descriptor DATA was reclaimed before HOLDS"
    );
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_mutations,
        before.retained_basis_registry_mutations
    );
    drop(holds);
    assert!(!runtime.history.has_detached_retention_obligations());
    assert_eq!(budget.charged_memory_bytes(), 0);
    // This genuine Memory owner is local evidence only, not Directory close.
    runtime.prepare_close().unwrap().finish();
}
