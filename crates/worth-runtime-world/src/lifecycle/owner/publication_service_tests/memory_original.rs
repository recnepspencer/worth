//! Resident original observations join genuine owner artifacts.
use super::*;
use crate::lifecycle::RuntimeWorldObservationService;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn older_memory_original_survives_an_unrelated_unsealed_publication() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().unwrap();
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let first = ready_relational(&fixture, &owner, &expected, "original-memory-event");
    let first = first
        .capture_memory_original(
            &execution,
            &owner.state.history,
            &owner.state.relational.basis_port(),
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(performed) =
        first.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    else {
        panic!("the first actual publication performs");
    };
    let receipt = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    let original_head = performed.new_product_head().clone();
    drop(performed.consume());
    drop(expected);
    let expected = owner.observe_root_product_branch().unwrap();
    let later = ready_relational(
        &fixture,
        &owner,
        &expected,
        "unrelated-unsealed-memory-event",
    );
    let injection = crate::branch::publication_unwind::arm_materialized();
    assert!(catch_unwind(AssertUnwindSafe(
        || later.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    ))
    .is_err());
    drop(injection);
    assert_eq!(cell.atomic_snapshot(), original_head);
    assert_eq!(owner.state.history.len(), 3);
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let native = fixture
        .relational_runtime()
        .durability_authority()
        .observe_memory_commit(&execution, &receipt)
        .unwrap();
    let observed = owner
        .observe_memory_original_publication(&execution, native)
        .unwrap();
    assert_eq!(
        observed.after().selected_commit(),
        original_head.selected_commit().ordinal()
    );
    assert_eq!(
        observed.after().generation(),
        original_head.reference_generation().get()
    );
    assert_eq!(observed.native().receipt(), &receipt);
    assert_eq!(
        owner.state.history.len(),
        3,
        "a read cannot republish or remove the unsealed event"
    );
}

#[test]
fn memory_original_remains_readable_after_its_live_history_is_retired() {
    let (fixture, owner, expected) = setup();
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let cell = owner.state.branches.root_cell().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "retired-original");
    let ready = ready
        .capture_memory_original(
            &execution,
            &owner.state.history,
            &owner.state.relational.basis_port(),
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(first) =
        ready.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    else {
        panic!("actual original performs")
    };
    let receipt = first
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    let original = first.new_product_head().selected_commit().clone();
    let generation = first.new_product_head().reference_generation().get();
    drop(first.consume());
    drop(expected);
    let expected = owner.observe_root_product_branch().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "later-memory-event");
    let ready = ready
        .capture_memory_original(
            &execution,
            &owner.state.history,
            &owner.state.relational.basis_port(),
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(later) =
        ready.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    else {
        panic!("later actual publication performs")
    };
    let keep = later.new_product_head().selected_commit().clone();
    drop(later.consume());
    drop(expected);
    drop(
        owner
            .state
            .history
            .retire_unprotected_history(&keep)
            .unwrap(),
    );
    assert!(
        owner.state.history.lookup(&original).is_none(),
        "the original live World history really retired"
    );
    let native = fixture
        .relational_runtime()
        .durability_authority()
        .observe_memory_commit(&execution, &receipt)
        .unwrap();
    let observed = owner
        .observe_memory_original_publication(&execution, native)
        .unwrap();
    assert_eq!(observed.after().selected_commit(), original.ordinal());
    assert_eq!(observed.after().generation(), generation);
    assert_eq!(observed.native().receipt(), &receipt);
    let port = owner.state.relational.basis_port();
    let recorded = port
        .export_memory_component(&execution, observed.after_component())
        .unwrap();
    let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(component) =
        recorded.target()
    else {
        panic!("actual original is canonical")
    };
    assert_eq!(component.receipt(), &receipt);
}
