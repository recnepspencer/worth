//! Prepare one genuine intent and durable original World open, before Native.
use super::*;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_relational::facade::runtime::{
    CustomInvariantRegistration, PreparedRelationalHomeProgramContracts,
};
use worth_runtime_world::facade::{
    RuntimeWorldCancellationSource, WorldOpenAttemptOutcome, WorldOpenedPublicationAttempt,
};
use worth_store_application::{
    AdmittedApplicationHomeProgramStep, ApplicationHomeTransitionIntentOutcome as Outcome,
};
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

pub(super) fn contracts(
    fixture: &WorldHomeFixture,
    execution: &ExecutionRequest<'_, '_>,
    evaluated: Arc<AtomicUsize>,
) -> PreparedRelationalHomeProgramContracts {
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    native
        .prepare_home_program_contracts(
            execution,
            source.basis().relational_basis(),
            native.config().schema.registry.clone(),
            vec![
                CustomInvariantRegistration::new(super::transition_executor::TargetRule(evaluated))
                    .unwrap(),
            ],
            Vec::new(),
        )
        .unwrap()
}

pub(super) fn original_open(
    fixture: &mut WorldHomeFixture,
    contracts: &PreparedRelationalHomeProgramContracts,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    AdmittedApplicationHomeProgramStep,
    WorldOpenedPublicationAttempt,
) {
    original_open_steps(fixture, contracts, execution, 1)
}

pub(super) fn original_open_steps(
    fixture: &mut WorldHomeFixture,
    contracts: &PreparedRelationalHomeProgramContracts,
    execution: &ExecutionRequest<'_, '_>,
    steps: u32,
) -> (
    AdmittedApplicationHomeProgramStep,
    WorldOpenedPublicationAttempt,
) {
    let source = commit_intent_steps(fixture, contracts, execution, steps);
    let mut grant = fixture
        .home
        .admit_home_program_transition_step(0, &source, execution)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let reservation = grant.take_reservation().unwrap();
    let pending = fixture
        .home
        .world()
        .publication_port()
        .prepare_home_transition_step(
            execution,
            source,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            grant.plan(),
            grant.original(),
            reservation,
            grant.admission(),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let mut outcome = pending.advance(execution);
    let opened = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(value) => break value,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(execution),
            other => panic!("the assigned original open must commit: {other:?}"),
        }
    };
    (grant, opened)
}

pub(super) fn commit_intent(
    fixture: &mut WorldHomeFixture,
    contracts: &PreparedRelationalHomeProgramContracts,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_runtime_world::facade::ProductBranchObservation {
    commit_intent_steps(fixture, contracts, execution, 1)
}

fn commit_intent_steps(
    fixture: &mut WorldHomeFixture,
    contracts: &PreparedRelationalHomeProgramContracts,
    execution: &ExecutionRequest<'_, '_>,
    steps: u32,
) -> worth_runtime_world::facade::ProductBranchObservation {
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            steps,
            contracts.target_meaning(execution, 0).unwrap(),
            &crate::batches::transition_body_roster(execution, steps),
            execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), execution,
    ).unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(execution)
            .unwrap()
        {
            Outcome::Committed => break,
            Outcome::Pending => std::thread::yield_now(),
            other => panic!("the actual original intent must commit: {other:?}"),
        }
    }
    source
}

pub(super) fn reopen(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        super::transition_opening::successor_request(location),
        execution,
    )
    .expect("the same directory readmits its original pending transition")
}
