//! A candidate program commits only under the mutation binding whose handler
//! built it and the input that handler decided on, and one key and input never replay across bindings.
//!
//! `SetRetentionBinding` and `ReviewedSetRetentionBinding` act on one operation
//! with one input type. The guard tells them apart at commit; their key
//! namespaces keep a replay from crossing between them.

use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    primary_graph::{
        HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
        WorthQueryPrincipalResolutionMode,
    },
};

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::retention_entry::{
    ReviewedSetRetentionBinding, SetRetentionBinding, DOCUMENT_IDENTITY,
};
use crate::document_retention_model::schema::{
    DocumentIdentityField, DocumentPrincipalBinding, DocumentRetentionSchema, SetRetention,
    SetRetentionInput,
};

/// Admits one `SetRetention` operation on the branch, expanding to the
/// admission with the request's identities for `$binding`.
macro_rules! admit {
    ($runtime:expr, $branch:expr, $binding:ty, $key:expr, $input:expr) => {{
        let runtime = $runtime;
        let scope = request_scope();
        let external = authenticate_operator(runtime.installed_schema(), &scope);
        let selected = runtime.on_branch($branch).select().expect("branch selects");
        let principal_binding = runtime
            .installed_schema()
            .principal_binding(DocumentPrincipalBinding::reference())
            .expect("principal binding installs");
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                &external,
                &scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("operator resolves");
        let document = selected
            .resolve_entity(
                DocumentIdentityField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
                &scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("document resolves");
        let operation = runtime
            .installed_schema()
            .installed_operation(SetRetention::reference())
            .expect("operation installs");
        let admission = selected
            .authorize_operation(
                &principal,
                &document,
                &operation,
                Default::default(),
                &scope,
            )
            .expect("the operation admits");
        let identities =
            ApplicationMutationIdentities::<DocumentRetentionSchema, $binding>::encode(
                &$key, &$input,
            )
            .expect("key and input encode");
        (principal, admission, identities)
    }};
}

/// Runs the installed handler for `$binding` and yields its candidate program.
macro_rules! candidate {
    ($runtime:expr, $branch:expr, $binding:ty, $key:expr, $input:expr) => {{
        let (principal, admission, identities) = admit!($runtime, $branch, $binding, $key, $input);
        let HandlerResult::Completed(completed) = $runtime
            .with_application_advancement(
                &super::document_retention_model::operator_identity::request_scope(),
                |phase| {
                    $runtime.execute_mutation_handler::<$binding>(
                        &phase,
                        &identities,
                        principal.principal_identity(),
                        admission,
                    )
                },
            )
            .expect("the fixture policy admits its handler advancement")
            .expect("the handler runs")
        else {
            panic!("the handler must produce a candidate");
        };
        completed.into_parts().0
    }};
}

fn input(days: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: days,
    }
}

fn identities<'a, Binding>(
    key: &'a u64,
    input: &'a SetRetentionInput,
) -> ApplicationMutationIdentities<'a, DocumentRetentionSchema, Binding>
where
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
        IdempotencyKey = u64,
        Input = SetRetentionInput,
    >,
{
    ApplicationMutationIdentities::<DocumentRetentionSchema, Binding>::encode(key, input)
        .expect("key and input encode")
}

fn idempotency<Binding>(
    key: &u64,
    input: &SetRetentionInput,
) -> WorthQueryApplicationIdempotencyBinding
where
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
        IdempotencyKey = u64,
        Input = SetRetentionInput,
    >,
{
    WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities::<Binding>(
        key, input,
    ))
}

#[test]
fn a_program_built_by_one_binding_cannot_commit_under_another() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let before = read_retention(runtime, branch);
    let (key, request) = (7_001_u64, input(before + 1));

    let program = candidate!(runtime, branch, ReviewedSetRetentionBinding, key, request);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &request),
        std::convert::identity,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("a program must not commit under a binding other than its handler's");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::MutationBindingMismatch
    );
    assert_eq!(
        read_retention(runtime, branch),
        before,
        "nothing was written"
    );

    let program = candidate!(runtime, branch, SetRetentionBinding, key, request);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &request),
        std::convert::identity,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the same request under its own binding commits: {outcome:?}"
    );
    assert_eq!(read_retention(runtime, branch), before + 1);
}

#[test]
fn one_key_and_input_do_not_replay_across_bindings() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let (key, request) = (7_002_u64, input(read_retention(runtime, branch) + 1));

    let program = candidate!(runtime, branch, SetRetentionBinding, key, request);
    assert!(matches!(
        application.compare_and_commit_program_action(
            program,
            &identities::<SetRetentionBinding>(&key, &request),
            std::convert::identity,
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let (_, admission, _) = admit!(runtime, branch, SetRetentionBinding, key, request);
    let resolve = |binding: WorthQueryApplicationIdempotencyBinding| {
        runtime
            .resolve_admitted_application_idempotency(&admission, binding)
            .expect("the key resolves")
            .into_resolution()
    };
    assert!(
        matches!(
            resolve(idempotency::<SetRetentionBinding>(&key, &request)),
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)
        ),
        "the binding that committed replays its receipt"
    );
    let other = resolve(idempotency::<ReviewedSetRetentionBinding>(&key, &request));
    assert!(
        matches!(other, WorthQueryApplicationIdempotencyResolution::Unseen),
        "another binding's key namespace never replays this receipt: {other:?}"
    );
}

#[test]
fn a_program_decided_on_one_input_cannot_commit_under_another_inputs_idempotency() {
    let application = publish_on_first_program();
    let runtime = application.runtime();
    let branch = application.current_world();
    let before = read_retention(runtime, branch);
    let key = 7_003_u64;
    let (decided, requested) = (input(before + 1), input(before + 2));

    // The handler decides on `decided`; the caller-built idempotency binding
    // names `requested`, so the program and the key describe different intents.
    let program = candidate!(runtime, branch, SetRetentionBinding, key, decided);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &requested),
        std::convert::identity,
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("a program must not commit under another input's idempotency binding");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::MutationInputMismatch
    );
    assert_eq!(
        read_retention(runtime, branch),
        before,
        "nothing was written"
    );

    let program = candidate!(runtime, branch, SetRetentionBinding, key, decided);
    let outcome = application.compare_and_commit_program_action(
        program,
        &identities::<SetRetentionBinding>(&key, &decided),
        std::convert::identity,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the same input under its own idempotency binding commits: {outcome:?}"
    );
    assert_eq!(read_retention(runtime, branch), before + 1);
}
