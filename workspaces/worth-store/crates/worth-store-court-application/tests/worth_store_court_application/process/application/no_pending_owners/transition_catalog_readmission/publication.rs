//! Produce the first real target occurrence under a two-step original intent.
use super::*;
pub(super) fn target(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    evaluated: Arc<AtomicUsize>,
) -> (
    RelationalCallerAssociation,
    worth_relational::facade::history::RelationalCommitReceipt,
) {
    let mut fixture = super::super::transition_opening::open_transition(location, execution);
    let contracts = original::contracts(&fixture, execution, evaluated);
    let (assigned, opened) = original::original_open_steps(&mut fixture, &contracts, execution, 2);
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let native = fixture.home.relational().runtime();
    let source = opened.expected_head();
    let mut transaction = native
        .begin_home_program_step(
            execution,
            source.basis().relational_basis(),
            RelationalHomeProgramStepContracts::Target {
                contracts: &contracts,
                transition: None,
            },
            assigned.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(assigned.plan().identity(), 0),
            assigned.original(),
            &association,
            assigned.admission(),
            worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
        )
        .unwrap();
    transaction
        .push_batch(crate::batches::create(0), execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(execution, native).unwrap();
    let prepared = opened.prepare_native(native, execution, proposal).unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let performed = fixture.publish(execution, prepared, &cancellation.token());
    let receipt = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    assert_eq!(performed.new_product_head().reference_generation().get(), 1);
    let mut consumed = performed.consume();
    let source = consumed.take_successor_observation().unwrap();
    drop((consumed, source, assigned, contracts));
    // The original two-step plan still owns its live reservation. This is an
    // interrupted Home, not a completed close; durable occurrence zero remains.
    drop(fixture);
    (association, receipt)
}
