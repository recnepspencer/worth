use super::program_fixture::admitted_program;
use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    IdentityExecutionSchema, ProgramRequiredInput, ProgramRequiredMutationBinding,
    ProgramRequiredOperation, ProgramRequiredSiblingBinding,
};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

type Identities<'a> =
    ApplicationMutationIdentities<'a, IdentityExecutionSchema, ProgramRequiredMutationBinding>;

#[test]
fn a_mutation_request_derives_its_identities_once_and_names_its_binding() {
    let key = "key-1".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = Identities::encode(&key, &input).expect("the request encodes");
    let named = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
    let unnamed = WorthQueryApplicationIdempotencyBinding::new(
        *identities.key_identity(),
        *identities.input_identity(),
    );

    assert_eq!(named.key_text(), unnamed.key_text());
    assert_eq!(named.intent_identity(), unnamed.intent_identity());
    assert_ne!(
        named.intent_text(),
        unnamed.intent_text(),
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
    assert_eq!(build("open").intent_text(), build("open").intent_text());
    assert_ne!(build("open").intent_text(), build("closed").intent_text());
    assert_eq!(build("open").key_text(), build("closed").key_text());
}

#[test]
fn two_bindings_sharing_key_namespace_operation_and_input_type_never_replay_each_other() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
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

    let account = resolved_account(&world, "open", &request);
    let first = admitted_program(&world, &principal, &account, &request, "committed");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            original,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let account = resolved_account(&world, "committed", &request);
    let retry = admitted_program(&world, &principal, &account, &request, "committed");
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                retry,
                original,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
            ),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "the same binding replays its own commit"
    );

    let crossed = admitted_program(&world, &principal, &account, &request, "committed");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            crossed,
            sibling,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
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
        .execute_mutation_handler_observing_contact::<ProgramRequiredSiblingBinding>(
            &identities,
            principal.principal_identity(),
            admission,
            || contacts += 1,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
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
        let principal = authenticated_principal(&world, &request);
        let account = resolved_account(&world, "open", &request);
        let operation = world
            .application
            .installed_schema()
            .installed_operation(ProgramRequiredOperation::reference())
            .unwrap();
        let admission = world
            .selected_product()
            .authorize_operation(
                &principal,
                &account,
                &operation,
                TypedMutationPreconditions::new(),
                &request,
            )
            .unwrap();
        let key = "handler-contact".to_owned();
        let input = ProgramRequiredInput::new(status);
        let identities = Identities::encode(&key, &input).unwrap();
        let mut contacts = 0;
        let outcome = world
            .application
            .execute_mutation_handler_observing_contact::<ProgramRequiredMutationBinding>(
                &identities,
                principal.principal_identity(),
                admission,
                || contacts += 1,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        assert_eq!(contacts, 1, "the real decide boundary ran for {status}");
        assert!(
            outcome.is_err(),
            "a later execution denial preserves the actual handler contact"
        );
    }
}
