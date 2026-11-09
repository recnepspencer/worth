//! The same real Home promotes only after its original positioned publication.
use super::target;
mod installation;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_relational::facade::{
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::{home_program_transition_body_digest, RelationalHomeProgramStepContracts},
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::{
    ApplicationHomeTransitionCompletionDenial, ApplicationHomeTransitionFinishDenial,
    ApplicationHomeTransitionIntentOutcome,
};

#[test]
fn home_binding_promotes_the_original_receiver_only_after_all_actual_steps() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let mut fixture =
        super::super::transition_opening::open_transition(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let original_binding = fixture.home.binding().clone();
    let refused = fixture
        .home
        .readmit_no_pending_owners(&execution)
        .err()
        .expect("a requested transition cannot grant service");
    assert!(matches!(
        refused.reason(),
        worth_store_application::ApplicationHomeOwnerAdmissionDenial::ProgramTransitionRequired
    ));
    fixture.home = refused.into_home();
    let contracts = fixture
        .home
        .relational()
        .runtime()
        .prepare_home_program_contracts(
            &execution,
            source.basis().relational_basis(),
            target::registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
        .with_receiver_configuration(
            &execution,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE).resolve(),
        )
        .unwrap();
    let history = fixture
        .home
        .relational()
        .runtime()
        .history()
        .historical_latest_commit();
    let matching_preparation =
        installation::matching_preparation(&fixture.home, &source, &execution);
    assert_eq!(
        matching_preparation.catalog_digest(),
        contracts.catalog_digest()
    );
    assert!(matches!(
        fixture
            .home
            .finish_home_program_transition(&source, &contracts, &execution),
        Err(ApplicationHomeTransitionFinishDenial::Completion(
            ApplicationHomeTransitionCompletionDenial::Incomplete
        ))
    ));
    assert_eq!(fixture.home.binding(), &original_binding);
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .history()
            .historical_latest_commit(),
        history
    );
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
            HomeProgramTransitionPlanDigest::from_recorded([117; 32]),
            1,
            contracts.target_meaning(&execution, 0).unwrap(),
            &[digest],
            &execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution).unwrap();
    let progress_limit = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < progress_limit,
            "the original intent must settle"
        );
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(&execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            other => panic!("original intent refused: {other:?}"),
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
        assert!(
            std::time::Instant::now() < progress_limit,
            "the original World open must settle"
        );
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
            other @ (WorldOpenAttemptOutcome::Unresolved(_)
            | WorldOpenAttemptOutcome::EvidenceRequired(_)
            | WorldOpenAttemptOutcome::Refused(_)) => {
                panic!("original World open refused: {other:?}")
            }
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
    let candidate = native
        .prepare_validated_proposal(&execution, proposal)
        .unwrap();
    let prepared = opened
        .complete_query_native_preparation(candidate, &execution)
        .unwrap();
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    // Even an actual durable Target is not permission to bind until its real
    // performed delivery has crossed the Home completion owner.
    let current = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert!(matches!(
        fixture
            .home
            .finish_home_program_transition(&current, &contracts, &execution),
        Err(ApplicationHomeTransitionFinishDenial::Completion(
            ApplicationHomeTransitionCompletionDenial::Incomplete
        ))
    ));
    assert_eq!(fixture.home.binding(), &original_binding);
    let refused = fixture
        .home
        .readmit_no_pending_owners(&execution)
        .err()
        .expect("durable steps without a binding still refuse service");
    assert!(matches!(
        refused.reason(),
        worth_store_application::ApplicationHomeOwnerAdmissionDenial::JournalNotEmpty {
            journal: "home.transitions",
            ..
        }
    ));
    fixture.home = refused.into_home();
    fixture
        .home
        .accept_home_program_transition_step(performed, &execution)
        .unwrap();
    let receipt = fixture
        .home
        .finish_home_program_transition(&current, &contracts, &execution)
        .unwrap();
    assert_eq!(
        receipt.runtime_instance_id(),
        fixture.home.relational().runtime().runtime_instance_id()
    );
    assert_eq!(
        fixture.home.relational().runtime().config().schema.registry,
        target::registry()
    );
    assert_ne!(fixture.home.binding(), &original_binding);
    let installed_binding = fixture.home.binding().clone();
    let installation = installation::current(&fixture.home, &original_binding);
    installation::admit_exact_target(
        &fixture.home,
        &contracts,
        &matching_preparation,
        &receipt,
        &installation,
    );
    fixture.home = fixture
        .home
        .readmit_no_pending_owners(&execution)
        .unwrap_or_else(|cause| panic!("the actual acknowledged binding admits service: {cause:?}"))
        .into_no_pending_home();
    // Actual ordinary publication after binding must use B's writable receiver.
    let native = fixture.home.relational().runtime();
    let view = native
        .read_truth()
        .project_observation(&current.basis().relational_basis().observation())
        .unwrap();
    let records = view
        .bounded_entities_of_kind(
            &execution,
            worth_relational::facade::identity::KindId(2000),
            100,
        )
        .unwrap()
        .unwrap();
    assert_eq!(records.records().len(), 1);
    let id = records.records()[0].entity_id;
    drop((records, view));
    let mut batch = target::body();
    let worth_relational::facade::transactions::MutationIntent::Create(
        worth_relational::facade::transactions::CreateIntent::Entity(entity),
    ) = batch.intents.pop().unwrap()
    else {
        unreachable!()
    };
    let field = entity.fields.locators().next().unwrap().clone();
    let fields = worth_relational::facade::transactions::AspectFieldPatch::from_locator(
        field,
        worth_foundational::facade::AspectValue::String("after-B".into()),
    );
    let update = worth_relational::facade::transactions::WorkerIntentBatch::new("ordinary-B-write")
        .push(
            worth_relational::facade::transactions::MutationIntent::Entity(
                worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                    worth_relational::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id: id,
                        fields,
                    },
                ),
            ),
        );
    let prepared =
        fixture.prepare_publication(&execution, current.clone(), update, &cancellation.token());
    drop(
        fixture
            .publish(&execution, prepared, &cancellation.token())
            .consume(),
    );
    let latest = fixture
        .home
        .relational()
        .runtime()
        .history()
        .historical_latest_commit();
    drop((source, current, grant, receipt));
    super::super::close(fixture.home, &execution);
    let definition = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
        .schema_registry(target::registry())
        .resolve();
    let request = worth_store_application::ApplicationHomeRequest::new(directory.path().into(), definition,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(),
        worth_foundational::facade::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111").unwrap());
    let reopened = worth_store_testkit_application_oracle::world_home::open(request, &execution)
        .unwrap_or_else(|cause| panic!("ordinary B reopen uses the original Home: {cause:?}"));
    assert_eq!(reopened.home.binding(), &installed_binding);
    installation::reopened(&reopened.home, installation);
    assert_eq!(
        reopened
            .home
            .relational()
            .runtime()
            .history()
            .historical_latest_commit(),
        latest
    );
    assert_eq!(
        reopened
            .home
            .relational()
            .runtime()
            .config()
            .schema
            .registry,
        target::registry()
    );
    assert!(reopened.home.home_program_transition_intent().is_none());
    {
        let source = reopened
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        let native = reopened.home.relational().runtime();
        let view = native
            .read_truth()
            .project_observation(&source.basis().relational_basis().observation())
            .unwrap();
        let rows = view
            .bounded_entities_of_kind(
                &execution,
                worth_relational::facade::identity::KindId(2000),
                100,
            )
            .unwrap()
            .unwrap();
        let [row] = rows.records() else {
            panic!("the same authored Target entity must survive B close/reopen");
        };
        assert_eq!(row.entity_id, id);
        let state = row.authoritative_aspect_state.as_ref().unwrap();
        let aspect = state
            .get(&worth_foundational::facade::AspectKey::new("target-label").unwrap())
            .unwrap();
        let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) =
            aspect.view()
        else {
            panic!("actual Target contract is a struct");
        };
        assert_eq!(
            fields.get(&crate::batches::field_key("label")),
            Some(&worth_foundational::facade::AspectValue::String(
                "after-B".into()
            ))
        );
    }
    let home = reopened
        .home
        .readmit_no_pending_owners(&execution)
        .unwrap_or_else(|cause| {
            panic!("the completed original intent admits ordinary B service: {cause:?}")
        })
        .into_no_pending_home();
    super::super::close(home, &execution);
    // Prepared contract meaning itself carries no live Home/Native Source port.
    drop(contracts);
}
