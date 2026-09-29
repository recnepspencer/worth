//! The public program-owner commit door cannot bypass workflow authority.

use super::*;
use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    primary_graph::{HandlerResult, WorthQueryPrincipalResolutionMode},
};

use super::super::document_retention_model::retention_entry::SetRetentionBinding;
use super::super::document_retention_model::schema::{
    DocumentIdentityField, DocumentRetentionSchema, SetRetention, SetRetentionInput,
};

fn reviewed_identities<'a>(
    key: &'a u64,
    input: &'a SetRetentionInput,
) -> ApplicationMutationIdentities<'a, DocumentRetentionSchema, ReviewedSetRetentionBinding> {
    ApplicationMutationIdentities::encode(key, input).expect("key and input must encode")
}

#[test]
fn guarded_action_cannot_commit_through_public_program_owner_without_workflow_authority() {
    let application =
        super::super::document_retention_model::host::publish_workflow_on_first_program();
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
            super::super::document_retention_model::schema::DocumentPrincipalBinding::reference(),
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
            .execute_mutation_handler::<ReviewedSetRetentionBinding>(
                &reviewed_identities(&key, &input),
                principal.principal_identity(),
                admission,
            )
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

#[test]
fn one_approval_transition_cannot_commit_twice_under_different_client_keys() {
    use worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowAdvanceAdapter;

    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 960);
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            970,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 971).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an approved operation, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &scope);
    let selected = runtime.on_branch(instance.branch()).select().unwrap();
    let principal_binding = runtime
        .installed_schema()
        .principal_binding(
            super::super::document_retention_model::schema::DocumentPrincipalBinding::reference(),
        )
        .unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            &principal_binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let document = selected
        .resolve_entity(
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
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
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
            .unwrap();
        let HandlerResult::Completed(completed) = runtime
            .execute_mutation_handler::<ReviewedSetRetentionBinding>(
                &reviewed_identities(&key, &input),
                principal.principal_identity(),
                admission,
            )
            .unwrap()
        else {
            panic!("the reviewed handler must produce a candidate");
        };
        completed.into_parts().0
    };
    let authority = required
        .authority_slot()
        .take()
        .expect("the owner issued one operation authority");
    let extend = |idempotency| {
        WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency(
            idempotency,
            required.transition_identity_bytes(),
        )
    };
    let sibling_key = 974_u64;
    let sibling_admission = selected
        .authorize_operation(
            &principal,
            &document,
            &operation,
            Default::default(),
            &scope,
        )
        .unwrap();
    let HandlerResult::Completed(sibling) = runtime
        .execute_mutation_handler::<
            super::super::document_retention_model::retention_entry::SetRetentionBinding,
        >(
            &ApplicationMutationIdentities::<DocumentRetentionSchema, SetRetentionBinding>::encode(
                &sibling_key,
                &input,
            )
            .unwrap(),
            principal.principal_identity(),
            sibling_admission,
        )
        .unwrap()
    else {
        panic!("the ordinary sibling handler must produce a candidate");
    };
    let sibling_program = sibling
        .into_parts()
        .0
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let relabeled = application
        .program_runtime()
        .compare_and_commit_program_action(
            sibling_program,
            &reviewed_identities(&sibling_key, &input),
            extend,
        );
    assert!(matches!(
        relabeled,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired
    ));
    assert_eq!(read_retention(runtime, instance.branch()), SEED_RETENTION);
    let first_program = candidate(972)
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let second_program = candidate(973)
        .bind_workflow_operation_authority(&authority)
        .unwrap();
    let first = application
        .program_runtime()
        .compare_and_commit_program_action(
            first_program,
            &reviewed_identities(&972, &input),
            extend,
        );
    assert!(matches!(
        first,
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let second = application
        .program_runtime()
        .compare_and_commit_program_action(
            second_program,
            &reviewed_identities(&973, &input),
            extend,
        );
    assert!(matches!(
        second,
        WorthQueryApplicationCommitOutcome::Denied(denial)
            if denial.kind() == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    ));
    assert_eq!(read_retention(runtime, instance.branch()), 8);
}
