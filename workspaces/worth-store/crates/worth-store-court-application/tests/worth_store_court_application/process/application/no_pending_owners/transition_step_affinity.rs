//! A genuine Home grant cannot authorize a separately issued World attempt.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_relational::facade::{
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::{CustomInvariantRegistration, RelationalHomeProgramContractDenial},
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::ApplicationHomeTransitionIntentOutcome as Outcome;

#[test]
fn a_committed_transition_grant_refuses_another_world_attempt_before_target_execution() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let evaluated = Arc::new(AtomicUsize::new(0));
    let native = fixture.home.relational().runtime();
    let contracts = native
        .prepare_home_program_contracts(
            &execution,
            source.basis().relational_basis(),
            native.config().schema.registry.clone(),
            vec![
                CustomInvariantRegistration::new(super::transition_executor::TargetRule(
                    evaluated.clone(),
                ))
                .unwrap(),
            ],
            Vec::new(),
        )
        .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            1,
            contracts.target_meaning(&execution, 0).unwrap(),
            &crate::batches::transition_body_roster(&execution, 1),
            &execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution,
    ).unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(&execution)
            .unwrap()
        {
            Outcome::Committed => break,
            Outcome::Pending => std::thread::yield_now(),
            other => panic!("the genuine original intent must commit: {other:?}"),
        }
    }
    let grant = fixture
        .home
        .admit_home_program_transition_step(0, &source, &execution)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut outcome = fixture
        .home
        .world()
        .publication_port()
        .open_publication_without_signal(
            &execution,
            source.clone(),
            fixture.admit_initial_program_source(&source),
            RelationalTransactionIntent::ordinary(),
            &cancellation.token(),
            None,
        );
    let other = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
            other => panic!("the control is a real, independently opened attempt: {other:?}"),
        }
    };
    assert_ne!(
        other.attempt_identity().ordinal(),
        grant.original().attempt().ordinal()
    );
    let refused = fixture.home.relational().runtime().begin_home_program_step(
        &execution,
        source.basis().relational_basis(),
        worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
            contracts: &contracts,
            transition: None,
        },
        grant.plan(),
        worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
            grant.plan().identity(),
            grant.plan().target_meaning().step(),
        ),
        grant.original(),
        &RelationalCallerAssociation::from_bytes(other.caller_association_bytes()).unwrap(),
        grant.admission(),
        worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: None,
            maximum_derived_rows: None,
        },
    );
    assert!(matches!(
        refused,
        Err(RelationalHomeProgramContractDenial::Admission(
            worth_proof::HomeProgramTransitionAdmissionDenial::CallerAssociationMismatch
        ))
    ));
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    // Keep actual open custody until abandonment; no fabricated no-effect or close.
    drop(other.retain());
    drop(grant);
    drop(contracts);
    drop(source);
    drop(fixture);
}
