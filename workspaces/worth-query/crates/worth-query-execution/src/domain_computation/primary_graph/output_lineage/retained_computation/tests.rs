//! Ledger refusal and unmeasured state are different record absences.

use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    sealed_run_for_lineage_test, WorthQueryPartitionedComputationFullCause as Cause,
};

#[test]
fn overflow_is_unmeasured_and_a_real_ledger_refusal_is_evicted() {
    let mut run = sealed_run_for_lineage_test();
    run.state.overflow_bytes_for_test();
    let lineage = WorthQueryApplicationOutputLineage::default();
    assert!(matches!(
        lineage.retain_computation(run, None, None),
        RecordedComputation::Absent(PriorAbsence::Unmeasured)
    ));
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.retention = super::super::retained_capacity::LineageRetentionLedger::new(0);
    let recorded = lineage.retain_computation(sealed_run_for_lineage_test(), None, None);
    assert!(matches!(
        recorded,
        RecordedComputation::Absent(PriorAbsence::Evicted)
    ));
    assert_eq!(PriorAbsence::Evicted.full_cause(), Cause::Evicted);
}

#[test]
fn moving_exact_displaced_state_leaves_the_moved_absence() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), 1, 1)),
        output_binding: TypeId::of::<()>(),
    };
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let lineage = WorthQueryApplicationOutputLineage::default();
    let computation = lineage.retain_computation(sealed_run_for_lineage_test(), None, None);
    let RecordedComputation::Retained { state, .. } = &computation else {
        panic!("measured state is retained");
    };
    let state = Arc::clone(state);
    let bytes = state.retained_bytes().unwrap();
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let other_identity = RecordedSettlementIdentity::retain(&source, coordinate, 1);
    let other = lineage.retain_computation(sealed_run_for_lineage_test(), None, None);
    let RecordedComputation::Retained {
        state: other_state, ..
    } = &other
    else {
        panic!("second state is retained");
    };
    let record = RecordedOutput {
        computation_source: super::super::computation_source::ComputationSourceEvidence::for_test(
            false,
        ),
        performed_origin: None,
        _retained_capacity: None,
        consumed_outputs: Arc::from([]),
        completed_handler_facts: None,
        completed_decision_reuse: None,
        prepared_input_reuse_key: None,
        native_output_witness: OnceLock::new(),
        mutable: std::sync::Mutex::new(super::super::RecordedOutputMutable::new(
            None,
            None,
            None,
            computation,
        )),
        settlement_identity: Arc::clone(&identity),
        correspondence: Arc::new(
            super::super::WorthQueryApplicationOutputCorrespondence::default(),
        ),
        source_identity: None,
        source_partition_identity: None,
        producer_dependency_identity: None,
        idempotency_key_identity: [0; 32],
    };
    let prior = PriorComputationRecord {
        cell: Arc::new(OnceLock::from(record)),
    };
    assert!(
        prior.take_capacity(&state, &other_identity).is_none(),
        "another record is never displaced"
    );
    assert!(
        prior.take_capacity(other_state, &identity).is_none(),
        "another state allocation does not establish pointer identity"
    );
    let capacity = prior
        .take_capacity(&state, &identity)
        .expect("the exact displaced state's custody moves");
    assert_eq!(capacity.bytes(), bytes);
    assert!(matches!(
        prior
            .cell
            .get()
            .unwrap()
            .mutable
            .lock()
            .unwrap()
            .computation,
        RecordedComputation::Absent(PriorAbsence::Moved)
    ));
    assert!(
        prior.take_capacity(&state, &identity).is_none(),
        "custody moves once"
    );
    drop(state);
    drop(capacity);
}
