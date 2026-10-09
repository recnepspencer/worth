//! Public program commits cannot bypass workflow authority.
use super::*;

#[test]
fn guarded_action_cannot_commit_through_public_program_owner_without_workflow_authority() {
    let application =
        super::super::super::document_retention_model::host::publish_workflow_on_first_program();
    let runtime = application.runtime();
    let branch = application.program_runtime().current_world();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let selected = runtime
        .on_branch(branch)
        .select()
        .expect("branch must select");
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::super::document_retention_model::schema::DocumentPrincipalBinding::reference(),
        )
        .expect("principal binding must install");
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("operator must resolve");
    let document = selected
        .resolve_entity(
            DocumentIdentityField::reference(),
            DOCUMENT_IDENTITY.to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("document must resolve");
    let operation = runtime
        .installed_schema()
        .installed_operation(SetRetention::reference())
        .expect("operation must install");
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: SEED_RETENTION + 1,
    };
    let candidate = |key: u64| {
        let admission = selected
            .authorize_operation(
                &principal,
                &document,
                &operation,
                Default::default(),
                &scope,
            )
            .expect("ordinary operation admission must succeed");
        let HandlerResult::Completed(completed) = runtime
            .with_application_advancement(&scope, |phase| {
                runtime.execute_mutation_handler::<ReviewedSetRetentionBinding>(
                    &phase,
                    &reviewed_identities(&key, &input),
                    principal.principal_identity(),
                    admission,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .expect("the fixture policy admits its handler advancement")
            .expect("handler may prepare a candidate")
        else {
            panic!("handler must produce a candidate");
        };
        completed.into_parts().0
    };
    let key = 950_u64;
    let outcome = application
        .program_runtime()
        .compare_and_commit_program_action(
            candidate(key),
            &reviewed_identities(&key, &input),
            std::convert::identity,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    let admission = application
        .program_runtime()
        .admit_program_operation::<SetRetention>();
    assert!(matches!(
        admission,
        Err(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            branch,
            SEED_RETENTION + 1,
            key + 2,
        )),
        RetentionVerdict::Performed(SEED_RETENTION + 1),
        "the ordinary binding remains independently usable"
    );
}
