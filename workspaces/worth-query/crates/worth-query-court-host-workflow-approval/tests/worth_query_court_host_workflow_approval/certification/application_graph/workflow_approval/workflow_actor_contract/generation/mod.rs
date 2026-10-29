pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::workflow_actor_contract::generation::*;

#[test]
fn workflow_actor_contract_reused_principal_slot_is_not_the_original_generation() {
    let mut application = retain_workflow(
        publish(
            validated_first_program(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap(),
    );
    support_workflow_program::<ActorProgram>(&mut application);
    let target = *application
        .supported_program::<ActorProgram>()
        .unwrap()
        .installed_program()
        .revision();
    {
        let scope = request_scope();
        let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
        let programs = application.runtime().request(&principal, &scope).programs();
        let comparison = programs.compare(&target).unwrap();
        let prepared = programs.adopt(&comparison).unwrap().prepare(1024).unwrap();
        assert!(matches!(
            prepared.publish(application.runtime(), &principal, &scope),
            WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
        ));
    }
    // Bind against the vocabulary installed for the actually adopted program,
    // rather than the host's initial P0 vocabulary.
    let contract = application
        .with_host_advancement(|phase| {
            application
                .vocabulary()
                .workflow_spec_on(&phase, application.current_world())
                .unwrap()
                .bind_definition(reviewed_document_definition("actor-generation"))
                .unwrap()
        })
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let publication = application
        .runtime()
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&74_000_u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute();
    let definition = match publication {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("actor program publishes its real definition: {other:?}"),
    };
    let instance =
        super::super::instance_migration::started(start_instance(&application, definition, 74_001))
            .instance()
            .clone();
    let (proposal, required, _) =
        super::super::journey::approval_requirement(&application, instance.clone(), 74_010);
    let original = match propose_instance(&application, instance.clone(), 74_012).unwrap() {
        WorkflowProposalOutcome::Published(replay) => {
            assert!(replay.replayed());
            replay.receipt().committed_changes().workflow_actor_fields_for_test(proposal.entity_id(),
                worth_query_decl::facade::application_capability::ApplicationWorkflowRecordedRole::Proposer)
                .unwrap().unwrap()
        }
        other => panic!("original proposal remains exact native history: {other:?}"),
    };
    lifecycle(&application, false, 74_030);
    let reclaimed = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .run_actor_retention_pass_for_test(&phase)
                .unwrap()
        })
        .unwrap();
    assert!(
        reclaimed.0 > 0,
        "native retention must actually reclaim the retired principal; no policy/pin override"
    );
    lifecycle(&application, true, 74_031);
    // The same external/logical identity now resolves through its actual new
    // native mapping. It has fresh real grants and is a different generation.
    let runtime = application.runtime();
    let scope = request_scope();
    let successor = authenticate_operator(runtime.installed_schema(), &scope);
    let signing = runtime
        .request(&successor, &scope)
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&74_040_u64)
        .prepare_workflow_approval(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let intent = signing.authentication_intent().unwrap().clone();
    let event = super::super::super::document_retention_model::operator_identity::block_on(
        application
            .authentication()
            .authenticate((), &successor, intent, &scope),
    )
    .unwrap();
    let performed = match signing.sign(&event).unwrap().execute() {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("full-identity distinction must admit the genuine successor: {other:?}"),
    };
    let actual = performed.approval().unwrap().approver();
    assert_eq!(actual.partition_value(), original.partition_value());
    assert_eq!(
        actual.local_slot_value(),
        original.local_slot_value(),
        "the court requires actual slot reuse"
    );
    assert_ne!(actual.generation_value(), original.generation_value());
    assert_eq!(
        read_retention(runtime, runtime.current_world()),
        SEED_RETENTION
    );
}
