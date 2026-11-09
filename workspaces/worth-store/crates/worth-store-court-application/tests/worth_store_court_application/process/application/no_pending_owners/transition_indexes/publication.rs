//! Private definitions join the actual positioned Home Target effect.
use super::{target, Journey};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_relational::facade::{
    indexes::{DerivedIndexEntries, DerivedIndexMaintenanceBudget},
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
        super::super::transition_opening::open_transition(directory.path(), &execution);
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
        .prepare_home_program_definitions(&execution)
        .unwrap();
    assert_eq!(
        private.install(&execution, &old_definition).unwrap(),
        old_id
    );
    let target_id = private.install(&execution, &target_definition).unwrap();
    assert_eq!(
        target_id.0,
        old_id.0 + 1,
        "the owner assigns the next contiguous identity"
    );
    assert_eq!(
        private.install(&execution, &target_definition).unwrap(),
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
            &execution,
            source.basis().relational_basis(),
            target::registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
        .with_index_definitions(private.finish(&execution).unwrap())
        .unwrap();
    let proposed = target::proposed();
    let intent =
        RelationalTransactionIntent::ordinary().with_schema_transition(proposed.clone(), None);
    let body = target::body();
    let digest =
        home_program_transition_body_digest(&execution, &intent, std::slice::from_ref(&body))
            .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([83; 32]),
            1,
            contracts.target_meaning(&execution, 0).unwrap(),
            &[digest],
            &execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution).unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(&execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            other => panic!("real intent refused: {other:?}"),
        }
    }
    let mut grant = fixture
        .home
        .admit_home_program_transition_step(0, &source, &execution)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let reservation = grant.take_reservation().unwrap();
    let pending = fixture
        .home
        .world()
        .publication_port()
        .prepare_home_transition_step(
            &execution,
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
    let mut outcome = pending.advance(&execution);
    let opened = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
            other => panic!("original World open refused: {other:?}"),
        }
    };
    let native = fixture.home.relational().runtime();
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let mut transaction = native
        .begin_home_program_step(
            &execution,
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
        )
        .unwrap();
    transaction
        .push_batch(body, execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(&execution, native).unwrap();
    let (opened, proposal) = opened.bind_query_native_proposal(proposal).unwrap();
    let mut candidate = native
        .prepare_validated_proposal(&execution, proposal)
        .unwrap();
    let requested = match journey {
        Journey::IncompleteGenerations => &[][..],
        Journey::Publish | Journey::Abandon | Journey::ChangedCatalog => {
            std::slice::from_ref(&target_id)
        }
    };
    native
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            requested,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(100_000),
                maximum_cold_record_slots: Some(100),
                maximum_derived_rows: Some(100),
            },
            &execution,
        )
        .unwrap();
    assert!(native
        .index_access()
        .matching_definition(&target_definition)
        .is_none());
    let mut retained = None;
    match journey {
        Journey::Publish => {
            let prepared = opened
                .complete_query_native_preparation(candidate, &execution)
                .unwrap();
            let performed = fixture.publish(&execution, prepared, &cancellation.token());
            assert_eq!(performed.new_product_head().reference_generation().get(), 1);
            assert_eq!(
                native
                    .index_access()
                    .matching_definition(&old_definition)
                    .unwrap()
                    .index_id,
                old_id
            );
            assert_eq!(
                native
                    .index_access()
                    .matching_definition(&target_definition)
                    .unwrap()
                    .index_id,
                target_id
            );
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .unwrap();
            let observation = basis.observation();
            let generation = native
                .index_access()
                .read_published_generation_for_observation(&execution, target_id, &observation)
                .unwrap()
                .unwrap();
            let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
                panic!("actual B field generation");
            };
            assert_eq!(
                entries
                    .values()
                    .map(|entities| entities.len())
                    .sum::<usize>(),
                1
            );
            let receipt = native.history().historical_latest_commit().unwrap();
            let envelope = native
                .replay()
                .canonical_commit_envelope(&execution, receipt.commit_id)
                .unwrap()
                .unwrap();
            let wire: serde_json::Value =
                serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
            let definitions = wire["index_definition_delta"]["definitions"]
                .as_array()
                .unwrap();
            assert!(definitions
                .iter()
                .any(|definition| definition["index_id"] == target_id.0
                    && definition["name"] == "target-label"));
            let view = native
                .read_truth()
                .project_observation(&observation)
                .unwrap();
            let worth_relational::facade::indexes::DerivedIndexKind::EntityField { field_locator } =
                &target_definition.kind
            else {
                unreachable!();
            };
            let value = worth_foundational::facade::AspectValue::String("target-value".into());
            let lookup = native
                .index_access()
                .execute_bounded_entity_field_lookup(
                    worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup::new(
                        &view,
                        target_id,
                        worth_relational::facade::identity::KindId(2000),
                        field_locator,
                        &value,
                        10,
                    ),
                    worth_relational::facade::indexes::BoundedIndexParityMode::Production,
                    execution,
                )
                .unwrap();
            assert_eq!(lookup.candidate_entity_ids().len(), 1);
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
        Journey::ChangedCatalog => {
            let concurrent = target::definition("concurrent-name", "name", "name");
            native
                .index_authority()
                .register(concurrent.clone())
                .unwrap();
            assert!(matches!(
                native
                    .publication_port()
                    .compare_and_publish(&execution, candidate),
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
