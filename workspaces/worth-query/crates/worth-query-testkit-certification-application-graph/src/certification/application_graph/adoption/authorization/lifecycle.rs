//! Expired real authentication retains native adoption custody.
pub use super::*;
pub use std::num::NonZeroUsize;
pub use std::time::{Duration, SystemTime};
pub use worth_query_host::facade::admission::authenticated_principal::*;
pub use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryPrincipalResolutionDenialKind, WorthQueryPrincipalResolutionRefusalKind,
};
pub struct ExpiringOperator;
impl WorthQueryAuthenticationAdapter for ExpiringOperator {
    type Credential = Duration;
    fn configuration_identity(&self) -> &str {
        "worth-query-adoption-expiring-operator-v1"
    }
    fn validate<'a>(
        &'a self,
        lifetime: Duration,
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::operator_identity(),
                WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
                WorthQueryAuthenticationMethod::new("certification").unwrap(),
                now,
                now + lifetime,
                Vec::new(),
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}
pub fn expired_operator(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<
    worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
> {
    let adapter = admit_authentication_adapter(
        host.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
        ),
        ExpiringOperator,
    )
    .unwrap();
    let principal =
        worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::block_on(
            adapter.authenticate(Duration::from_millis(50), scope),
        )
        .unwrap();
    std::thread::sleep(
        principal
            .valid_until()
            .saturating_duration_since(std::time::Instant::now())
            + Duration::from_millis(1),
    );
    assert!(principal.is_expired());
    principal
}
pub fn expired(cause: WorthQueryProgramAdoptionAuthorizationDenial) {
    match cause {
        WorthQueryProgramAdoptionAuthorizationDenial::PrincipalResolution(denial) => assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::Refused(
                WorthQueryPrincipalResolutionRefusalKind::ExpiredAuthentication
            )
        ),
        other => panic!("actual expired authentication must refuse: {other:?}"),
    }
}
