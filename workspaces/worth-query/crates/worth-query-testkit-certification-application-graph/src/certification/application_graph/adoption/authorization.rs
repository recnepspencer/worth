//! Capability admission at the real branch adoption publication boundary.
pub use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryProgramAdoptionAuthorizationDenial,
};
pub use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome,
        WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
        WorthQueryBranchAdoptionPublicationOutcome,
    },
    application_installation::WorthQueryProgramOwner,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::{publish_on_first_program, DocumentRetentionRuntime},
    operator_identity::{authenticate_operator, request_scope},
    programs::{RetentionProgramP0, RetentionProgramP1},
    readback::read_retention,
    workflow::{WorkflowGrantStatusInput, WorkflowGrantStatusIntent},
};
pub fn revoke(host: &DocumentRetentionRuntime<RetentionProgramP0>) {
    change_grant_status(host, host.current_world(), "revoked", 91764001);
}
pub fn change_grant_status(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    status: &str,
    key: u64,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let intent = WorkflowGrantStatusIntent {
        input: WorkflowGrantStatusInput {
            grant_identity: "program-adoption-grant".to_owned(),
            status: status.to_owned(),
        },
    };
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(intent)
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        matches!(
            outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "native grant mutation must commit: {outcome:?}"
    );
}
pub fn permission_denied(cause: WorthQueryProgramAdoptionAuthorizationDenial) {
    match cause {
        WorthQueryProgramAdoptionAuthorizationDenial::Authorization(denial) => assert!(
            matches!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing)
            ),
            "capability must refuse: {denial:?}"
        ),
        other => panic!("the installed capability must decide, not setup: {other:?}"),
    }
}

#[path = "authorization/branch_set.rs"]
pub mod branch_set;
#[path = "authorization/selected_program.rs"]
pub mod selected_program;

#[path = "authorization/lifecycle.rs"]
pub mod lifecycle;

#[path = "authorization/progression_expiry.rs"]
pub mod progression_expiry;
