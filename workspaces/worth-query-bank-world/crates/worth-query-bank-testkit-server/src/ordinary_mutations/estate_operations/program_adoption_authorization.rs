//! Explicit deployment permission is independent of estate/payment permission.
pub use super::notify_death::fixture::{notification_world, NotificationFixture};
pub use crate::facade::support::request_scope;
pub use bank_domain::estate::DeathNoticeStatus;
pub use bank_server::{BankApplicationP1, BankApplicationP2};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramAdoptionPreparationDenial,
        WorthQueryBranchAdoptionPublicationOutcome,
    },
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
        WorthQueryProgramAdoptionAuthorizationDenial,
    },
};
pub fn permission_denied(
    denial: WorthQueryApplicationProgramAdoptionPreparationDenial,
    expected: WorthQueryOperationAuthorizationRefusalKind,
) {
    match denial {
        WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(
            WorthQueryProgramAdoptionAuthorizationDenial::Authorization(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(expected)
        ),
        other => panic!("the actual capability must refuse: {other:?}"),
    }
}
pub fn revoke(fixture: &NotificationFixture) {
    let actor = fixture.authenticate_specialist();
    let scope = request_scope();
    let runtime = &fixture.world.runtime;
    let operation=runtime.retain_program_deployment_revocation(&actor,bank_domain::schema::ProgramDeploymentRevocationInput { grant_identity:"specialist-p0-deployment".into() },91764011_u64,worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions::new(),&scope).unwrap_or_else(|cause| panic!("declared issuer revocation: {cause:?}"));
    let request = runtime.request(&actor, &scope);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        operation.advance(&request)
    else {
        panic!("recorded issuer revocation completes")
    };
    assert!(matches!(
        done.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("issuer cleanup: {cause:?}"));
}
