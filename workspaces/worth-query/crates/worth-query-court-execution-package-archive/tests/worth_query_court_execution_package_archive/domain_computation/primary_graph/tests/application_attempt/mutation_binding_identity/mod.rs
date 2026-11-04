use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredMutationBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredSiblingBinding;
use worth_query_execution::facade::domain_computation::MutationHandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub type Identities<'a> =
    ApplicationMutationIdentities<'a, IdentityExecutionSchema, ProgramRequiredMutationBinding>;

#[test]
fn a_mutation_request_derives_its_identities_once_and_names_its_binding() {
    let key = "key-1".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = Identities::encode(&key, &input).expect("the request encodes");
    let named = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
    let unnamed = WorthQueryApplicationIdempotencyBinding::new_for_test(
        *identities.key_identity(),
        *identities.input_identity(),
    );

    assert_eq!(named.key_text_for_test(), unnamed.key_text_for_test());
    assert_eq!(named.intent_identity(), unnamed.intent_identity());
    assert_ne!(
        named.intent_text_for_test(),
        unnamed.intent_text_for_test(),
        "naming the binding is part of the intent"
    );
    assert!(named.is_for_mutation_binding(ProgramRequiredMutationBinding::IDENTITY));
    assert!(!named.is_for_mutation_binding("another.binding"));
    assert!(!unnamed.is_for_mutation_binding(ProgramRequiredMutationBinding::IDENTITY));
}

#[test]
fn a_retry_names_the_same_intent_and_a_changed_input_does_not() {
    let key = "key-2".to_owned();
    let build = |status: &str| {
        let input = ProgramRequiredInput::new(status);
        let identities = Identities::encode(&key, &input).expect("the request encodes");
        WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)
    };
    assert_eq!(
        build("open").intent_text_for_test(),
        build("open").intent_text_for_test()
    );
    assert_ne!(
        build("open").intent_text_for_test(),
        build("closed").intent_text_for_test()
    );
    assert_eq!(
        build("open").key_text_for_test(),
        build("closed").key_text_for_test()
    );
}

#[test]
fn two_bindings_sharing_key_namespace_operation_and_input_type_never_replay_each_other() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let key = "shared-key".to_owned();
    let input = ProgramRequiredInput::new("open");
    let original = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
        &Identities::encode(&key, &input).expect("the request encodes"),
    );
    let sibling = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
        &ApplicationMutationIdentities::<IdentityExecutionSchema, ProgramRequiredSiblingBinding>::encode(
            &key, &input,
        )
        .expect("the request encodes"),
    );
    assert_eq!(original.key_identity(), sibling.key_identity());
    assert_eq!(original.intent_identity(), sibling.intent_identity());

    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            original,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "committed",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                original,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation
            ),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "the same binding replays its own commit"
    );

    let crossed = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            crossed,
            sibling,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("another binding must not replay a key committed under this one");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}

#[test]
fn a_binding_the_schema_never_installed_is_refused_instead_of_panicking() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let key = "never-installed".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        ProgramRequiredSiblingBinding,
    >::encode(&key, &input)
    .expect("the request encodes");

    let mut contacts = 0;
    let outcome = world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            world
                .application
                .execute_mutation_handler_report_observing_contact::<ProgramRequiredSiblingBinding>(
                    phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    || contacts += 1,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .into_outcome()
        })
        .expect("fixture owner admits its advancement");
    assert!(
        matches!(
            outcome,
            Err(MutationHandlerExecutionDenial::HandlerNotInstalled)
        ),
        "a binding with no installed handler is a typed refusal"
    );
    assert_eq!(contacts, 0, "entry refusal never contacts the handler");
}

#[test]
fn handler_contact_is_preserved_after_a_later_execution_denial() {
    for status in ["open", "missing"] {
        let world = installed_authorization_world(true);
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                authenticated_principal(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                resolved_account(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ProgramRequiredOperation::reference())
            .unwrap();
        let admission = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .selected_product(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    )
                    .authorize_operation(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &principal,
                        &account,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let key = "handler-contact".to_owned();
        let input = ProgramRequiredInput::new(status);
        let identities = Identities::encode(&key, &input).unwrap();
        let mut contacts = 0;
        let outcome = world
            .application
            .with_host_advancement(|active_phase| {
                let phase = &active_phase;
                world
                    .application
                    .execute_mutation_handler_report_observing_contact::<ProgramRequiredMutationBinding>(
                        phase,
                        &identities,
                        principal.principal_identity(),
                        admission,
                        || contacts += 1,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .into_outcome()
            })
            .expect("fixture owner admits its advancement");
        assert_eq!(contacts, 1, "the real decide boundary ran for {status}");
        assert!(
            outcome.is_err(),
            "a later execution denial preserves the actual handler contact"
        );
    }
}
