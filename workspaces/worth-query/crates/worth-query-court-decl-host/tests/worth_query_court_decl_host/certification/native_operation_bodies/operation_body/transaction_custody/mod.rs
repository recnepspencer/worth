//! Program transactions retain their application and branch custody gates.
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
        WorthQueryApplicationInvariantProjectionAuthority,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    },
    product::{WorthQueryAdmittedChange, WorthQueryProductTransactionCommitError},
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::{
    advanced_commit::*, read_retention, DOCUMENT_IDENTITY,
};

type Schema = DocumentRetentionSchema<8>;
type Program =
    WorthQueryApplicationEffectProgram<Schema, SetRetention, SetRetentionInput, Document>;

fn candidate(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
) -> Program {
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    assert!(!external.is_expired());
    runtime
        .with_application_advancement(&scope, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = runtime
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let document = selected
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = runtime
                .installed_schema()
                .installed_operation(SetRetention::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &document,
                    &operation,
                    Default::default(),
                    &scope,
                )
                .unwrap();
            admission
                .validate_current_authority()
                .expect("the genuine admission has fresh authentication");
            let (_, snapshot, _) = projection
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, target| {
                        assert_eq!(
                            reader
                                .decision_field(target, DocumentRetentionField::reference())
                                .unwrap(),
                            Some(7)
                        );
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = runtime
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    snapshot,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let mut effects = reads
                .complete_projected_dependencies(
                    &phase,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let target = effects.existing_entity(&document).unwrap();
            effects
                .write_field(&target, DocumentRetentionField::reference(), 8)
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}

fn idempotency() -> WorthQueryApplicationIdempotencyBinding {
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
        ..Default::default()
    };
    let identities =
        ApplicationMutationIdentities::<Schema, BodyBinding<8>>::encode(&918_u64, &input).unwrap();
    WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)
}

fn caller() -> worth_execution::SerialRequest {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
    );
    worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    )
}

#[test]
fn program_transaction_refuses_a_fresh_change_from_another_application() {
    let (source, projection) = install_with_projection::<8>();
    let (target, _) = install_with_projection::<8>();
    let program = candidate(source.runtime(), &projection);
    let caller = caller();
    let execution = worth_execution::ExecutionRequest::serial(&caller);
    assert_eq!(execution.consult(), Ok(()));
    let result = target
        .runtime()
        .on_branch(target.current_world())
        .transaction()
        .apply(WorthQueryAdmittedChange::new(program, idempotency()))
        .commit_for_program(
            &execution,
            target.admit_program_operation::<SetRetention>().unwrap(),
        );
    assert!(
        matches!(
            result,
            Err(WorthQueryProductTransactionCommitError::ApplicationMismatch)
        ),
        "{result:?}"
    );
    assert_eq!(read_retention(&source), 7);
    assert_eq!(read_retention(&target), 7);
}

#[test]
fn program_transaction_refuses_a_fresh_change_on_another_branch() {
    let (app, projection) = install_with_projection::<8>();
    let sibling = app
        .with_host_advancement(|phase| {
            app.runtime()
                .branches()
                .fork(app.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let program = candidate(app.runtime(), &projection);
    assert_ne!(app.current_world(), sibling);
    let caller = caller();
    let execution = worth_execution::ExecutionRequest::serial(&caller);
    assert_eq!(execution.consult(), Ok(()));
    let result = app
        .runtime()
        .on_branch(sibling)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(program, idempotency()))
        .commit_for_program(
            &execution,
            app.admit_program_operation::<SetRetention>().unwrap(),
        );
    assert!(
        matches!(
            result,
            Err(WorthQueryProductTransactionCommitError::BranchMismatch)
        ),
        "{result:?}"
    );
    assert_eq!(read_retention(&app), 7);
}
