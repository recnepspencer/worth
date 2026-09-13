use std::panic::{catch_unwind, AssertUnwindSafe};

use worth_signal::facade::branch::SignalBranchSnapshotReconstructionDenial;
use worth_signal::facade::{SignalError, SignalGraph, SignalRuntime};

type Runtime = SignalRuntime<(), (), (), (), ()>;

fn runtime() -> Runtime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

fn assert_panics(operation: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(operation)).is_err());
}

#[test]
fn sealed_non_main_selection_and_catalog_are_canonical_through_root_and_observer() {
    let mut runtime = runtime();
    let main = runtime.current_branch();
    let main_basis = runtime
        .observe_signal_branch_basis(main.clone())
        .expect("the bootstrap basis is admitted");
    let (child, child_basis) = runtime
        .fork_signal_branch("sealed-selected-child", &main_basis)
        .expect("the owner creates a non-main selection")
        .into_parts();
    runtime
        .switch_branch(child.clone())
        .expect("selection remains construction-time behavior before sealing");
    let (captured, captured_basis) = runtime
        .capture_signal_branch_snapshot(&child_basis)
        .expect("the selected child acquires a concrete head")
        .into_parts();

    let expected_selected = runtime.current_branch();
    let expected_known = runtime.known_branches();
    let expected_child = runtime
        .branch_handle(child.id)
        .expect("the child is catalogued before sealing");
    let expected_ancestry = runtime.branch_ancestry(child.id);
    let expected_head = Some(captured.snapshot().meta.snapshot_id);
    assert_eq!(runtime.branch_head_snapshot_id(child.id), expected_head);

    let services = runtime
        .owner_component_services()
        .expect("the non-main canonical partition seals");

    assert_eq!(runtime.current_branch(), expected_selected);
    assert_eq!(runtime.known_branches(), expected_known);
    assert_eq!(
        runtime.branch_handle(child.id),
        Some(expected_child.clone())
    );
    assert_eq!(runtime.branch_ancestry(child.id), expected_ancestry);
    assert_eq!(runtime.branch_head_snapshot_id(child.id), expected_head);

    let observer = runtime.observe();
    assert_eq!(observer.current_branch(), expected_selected);
    assert_eq!(observer.known_branches(), expected_known);
    assert_eq!(observer.branch_handle(child.id), Some(expected_child));
    assert_eq!(observer.branch_ancestry(child.id), expected_ancestry);
    assert_eq!(observer.branch_head_snapshot_id(child.id), expected_head);

    let selected_reference = services
        .basis_port()
        .issue_managed_branch_reference(&captured_basis)
        .expect("the selected child remains canonical through the basis port");
    let selected_before_denial = services
        .basis_port()
        .observe_current(&selected_reference)
        .expect("the selected child is observable before denied switching");
    let error = runtime
        .switch_branch(main)
        .expect_err("post-seal branch selection is explicitly unavailable");
    assert!(matches!(error, SignalError::InvalidInput { .. }));
    assert_eq!(runtime.current_branch(), expected_selected);
    assert_eq!(runtime.known_branches(), expected_known);
    let selected_after_denial = services
        .basis_port()
        .observe_current(&selected_reference)
        .expect("denied selection does not disturb the canonical child cell");
    assert_eq!(
        selected_after_denial.observation(),
        selected_before_denial.observation()
    );
}

#[test]
fn detached_construction_state_surfaces_panic_before_access_and_leave_owner_healthy() {
    let mut runtime = runtime();
    let branch = runtime.current_branch();
    let basis = runtime
        .observe_signal_branch_basis(branch)
        .expect("the bootstrap basis is admitted");
    let services = runtime
        .owner_component_services()
        .expect("the canonical partition seals");
    let basis_port = services.basis_port();
    let reference = basis_port
        .issue_managed_branch_reference(&basis)
        .expect("the canonical basis remains managed");
    let expected = basis_port
        .observe_current(&reference)
        .expect("the canonical cell is healthy before assertion checks");

    assert_panics(|| {
        let _ = runtime.graph();
    });
    assert_panics(|| {
        let _ = runtime.graph_mut();
    });
    assert_panics(|| {
        let _ = runtime.config();
    });
    assert_panics(|| {
        let _ = runtime.config_mut();
    });
    assert_panics(|| {
        let _ = runtime.validate_schema_bindings();
    });
    assert_panics(|| {
        let _ = runtime.validate_merge_semantics();
    });
    assert_panics(|| {
        let _ = runtime.derive_evaluation_strategy();
    });
    assert_panics(|| runtime.clear_live_branch_mutation_residue());
    assert_panics(|| {
        let _ = runtime.observe().graph();
    });
    assert_panics(|| {
        let _ = runtime.observe().materialize().graph();
    });
    assert_panics(|| {
        let _ = runtime.checkpoint();
    });
    assert_panics(|| {
        let _ = runtime.telemetry();
    });
    assert_panics(|| {
        let _ = runtime.resource_runtime_summary();
    });
    assert_panics(|| {
        let _ = runtime.resource_runtime_summary_read_report();
    });
    assert_panics(|| {
        let _ = runtime.latest_resource_branch_restore_report();
    });
    assert_panics(|| {
        let _ = runtime.latest_resource_observation_batch_report();
    });
    assert_panics(|| {
        let _ = runtime.temporal_wake_summary();
    });
    assert_panics(|| {
        let _ = runtime.temporal_frontier_snapshot();
    });
    assert_panics(|| {
        let _ = runtime.observe().checkpoint_record();
    });
    assert_panics(|| {
        let _ = runtime.observe().temporal_diagnostics_summary_now();
    });
    assert_panics(|| {
        let _ = runtime.event_bus_mut();
    });
    assert_panics(|| {
        let _ = runtime.observations_mut();
    });

    let observed = basis_port
        .observe_current(&reference)
        .expect("assertion unwinds leave the canonical owner and cell healthy");
    assert_eq!(observed.observation(), expected.observation());
    assert_eq!(runtime.current_branch().id, observed.branch_id());
}

#[test]
fn portable_reconstruction_is_non_pristine_after_owner_sealing() {
    let mut source = runtime();
    let source_basis = source
        .observe_signal_branch_basis(source.current_branch())
        .expect("the source basis is admitted");
    let portable = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("the source creates a portable snapshot")
        .into_parts()
        .0
        .into_snapshot();

    let mut target = runtime();
    let target_branch = target.current_branch();
    let expected = target
        .observe_signal_branch_basis(target_branch.clone())
        .expect("the pristine target basis is admitted");
    let services = target
        .owner_component_services()
        .expect("the target owner seals before reconstruction");

    assert!(matches!(
        target.reconstruct_signal_branch_snapshot(&expected, &portable),
        Err(SignalBranchSnapshotReconstructionDenial::NonPristineRuntime)
    ));
    let reference = services
        .basis_port()
        .issue_managed_branch_reference(&expected)
        .expect("the denied reconstruction preserves the sealed basis");
    let observed = services
        .basis_port()
        .observe_current(&reference)
        .expect("the denied reconstruction leaves canonical state healthy");
    assert_eq!(observed.branch_id(), target_branch.id);
    assert_eq!(observed.observation(), expected.observation());
}

#[path = "sealed_inspection.rs"]
mod sealed_inspection;
