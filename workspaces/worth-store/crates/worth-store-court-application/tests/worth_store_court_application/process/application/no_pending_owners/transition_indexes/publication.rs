//! Private definitions join the actual positioned Home Target effect.
mod committed_indexes;
use super::{target, Journey};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_relational::facade::{
    indexes::DerivedIndexMaintenanceBudget,
    mvcc::{
        RelationalCallerAssociation, RelationalPublicationDenial, RelationalPublicationOutcome,
        RelationalTransactionIntent,
    },
    runtime::{home_program_transition_body_digest, RelationalHomeProgramStepContracts},
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::ApplicationHomeTransitionIntentOutcome;

pub(super) fn run(
    journey: super::Journey,
    execution: &ExecutionRequest<'_, '_>,
) -> Option<worth_relational::facade::indexes::RetainedDerivedIndexDefinition> {
    let directory = tempfile::tempdir().unwrap();
    let mut fixture =
        super::super::transition_opening::open_transition(directory.path(), execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let old_definition = target::definition("original-name", "name", "name");
    let target_definition = target::definition("target-label", "target-label", "label");
    let old_id = native
        .index_authority()
        .register(old_definition.clone())
        .unwrap()
        .index_id;
    let before_count = native
        .index_access()
        .definition_lookup_snapshot()
        .definition_count();
    let before_history = native.history().historical_latest_commit();
    let mut private = native
        .index_authority()
        .prepare_home_program_definitions(execution)
        .unwrap();
    assert_eq!(private.install(execution, &old_definition).unwrap(), old_id);
    let target_id = private.install(execution, &target_definition).unwrap();
    assert_eq!(
        target_id.0,
        old_id.0 + 1,
        "the owner assigns the next contiguous identity"
    );
    assert_eq!(
        private.install(execution, &target_definition).unwrap(),
        target_id
    );
    assert_eq!(
        native
            .index_access()
            .definition_lookup_snapshot()
            .definition_count(),
        before_count
    );
    assert!(native
        .index_access()
        .matching_definition(&target_definition)
        .is_none());
    let contracts = native
        .prepare_home_program_contracts(
            execution,
            source.basis().relational_basis(),
            target::registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
        .with_index_definitions(private.finish(execution).unwrap())
        .unwrap();
    let proposed = target::proposed();
    let intent =
        RelationalTransactionIntent::ordinary().with_schema_transition(proposed.clone(), None);
    let body = target::body();
    let digest =
        home_program_transition_body_digest(execution, &intent, std::slice::from_ref(&body))
            .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([83; 32]),
            1,
            contracts.target_meaning(execution, 0).unwrap(),
            &[digest],
            execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), execution).unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            other => panic!("real intent refused: {other:?}"),
        }
    }
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
            source.clone(),
            fixture.admit_initial_program_source(&source),
            intent,
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
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(execution),
            other => panic!("original World open refused: {other:?}"),
        }
    };
    let native = fixture.home.relational().runtime();
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let mut transaction = native
        .begin_home_program_step(
            execution,
            source.basis().relational_basis(),
            RelationalHomeProgramStepContracts::Target {
                contracts: &contracts,
                transition: Some((proposed, None)),
            },
            grant.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
            grant.original(),
            &association,
            grant.admission(),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(if matches!(journey, Journey::MaintenanceBudgetDenied) {
                    0
                } else {
                    100_000
                }),
                maximum_cold_record_slots: Some(100),
                maximum_derived_rows: Some(100),
            },
        )
        .unwrap();
    transaction
        .push_batch(body, execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(execution, native).unwrap();
    let (opened, proposal) = opened.bind_query_native_proposal(proposal).unwrap();
    let preparation = native.prepare_validated_proposal(execution, proposal);
    if matches!(journey, Journey::MaintenanceBudgetDenied) {
        let cause = preparation.expect_err("the admitted zero budget must stop real preparation");
        let worth_relational::facade::transactions::TransactionCommitError::Publication {
            error,
            ..
        } = cause
        else {
            panic!("the real index budget must retain its original publication cause: {cause:?}");
        };
        let denial = error
            .index_maintenance_denial()
            .expect("original maintenance denial");
        assert_eq!(denial.kind, worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded);
        assert_eq!(
            denial.work.work_units, 0,
            "no semantic maintenance unit was admitted"
        );
        drop(opened.retain());
        assert_eq!(
            native
                .index_access()
                .definition_lookup_snapshot()
                .definition_count(),
            before_count
        );
        assert_eq!(
            native
                .index_access()
                .matching_definition(&old_definition)
                .unwrap()
                .index_id,
            old_id
        );
        assert!(native
            .index_access()
            .matching_definition(&target_definition)
            .is_none());
        assert_eq!(native.history().historical_latest_commit(), before_history);
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap(),
            source
        );
        drop((grant, contracts, source, fixture));
        return None;
    }
    let mut candidate = preparation.unwrap();
    if matches!(journey, Journey::IncompleteGenerations) {
        candidate
            .omit_prepared_index_generations_for_test()
            .expect("the fault must remove this candidate's automatically prepared batch");
    }
    assert!(native
        .index_access()
        .matching_definition(&target_definition)
        .is_none());
    let mut retained = None;
    match journey {
        Journey::Publish => {
            let prepared = opened
                .complete_query_native_preparation(candidate, execution)
                .unwrap();
            let performed = fixture.publish(execution, prepared, &cancellation.token());
            assert_eq!(performed.new_product_head().reference_generation().get(), 1);
            let lookup = committed_indexes::observe(
                native,
                execution,
                &old_definition,
                old_id,
                &target_definition,
                target_id,
            );
            retained = Some(lookup.retain_definition());
            drop(performed.consume());
        }
        Journey::Abandon => {
            drop(candidate);
            drop(opened.retain());
            assert_eq!(
                native
                    .index_access()
                    .definition_lookup_snapshot()
                    .definition_count(),
                before_count
            );
            assert!(native
                .index_access()
                .matching_definition(&target_definition)
                .is_none());
            assert_eq!(native.history().historical_latest_commit(), before_history);
            assert_eq!(
                fixture
                    .home
                    .world()
                    .observation_port()
                    .observe_root_product_branch()
                    .unwrap(),
                source
            );
        }
        Journey::IncompleteGenerations => {
            assert!(matches!(
                native
                    .publication_port()
                    .compare_and_publish(execution, candidate),
                RelationalPublicationOutcome::Denied(
                    RelationalPublicationDenial::HomeProgramIndexDefinitionsRequired
                )
            ));
            drop(opened.retain());
            assert!(native
                .index_access()
                .matching_definition(&target_definition)
                .is_none());
            assert_eq!(native.history().historical_latest_commit(), before_history);
            assert_eq!(
                fixture
                    .home
                    .world()
                    .observation_port()
                    .observe_root_product_branch()
                    .unwrap(),
                source
            );
        }
        Journey::MaintenanceBudgetDenied => unreachable!(
            /* Proof: The zero-budget journey returned immediately after checking its preparation denial. */
            "zero-budget journey cannot publish"
        ),
        Journey::ChangedCatalog => {
            let concurrent = target::definition("concurrent-name", "name", "name");
            native
                .index_authority()
                .register(concurrent.clone())
                .unwrap();
            assert!(matches!(
                native
                    .publication_port()
                    .compare_and_publish(execution, candidate),
                RelationalPublicationOutcome::Denied(
                    RelationalPublicationDenial::StaleIndexDefinitions
                )
            ));
            drop(opened.retain());
            assert!(native
                .index_access()
                .matching_definition(&concurrent)
                .is_some());
            assert!(native
                .index_access()
                .matching_definition(&target_definition)
                .is_none());
            assert_eq!(native.history().historical_latest_commit(), before_history);
            assert_eq!(
                fixture
                    .home
                    .world()
                    .observation_port()
                    .observe_root_product_branch()
                    .unwrap(),
                source
            );
        }
    }
    // No final binding or transition resume is claimed. The actual owner keeps
    // every submitted open and canonical effect through abandonment.
    drop((grant, contracts, source, fixture));
    retained
}
