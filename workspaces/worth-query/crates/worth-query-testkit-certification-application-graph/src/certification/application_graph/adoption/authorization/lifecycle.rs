//! Expired real authentication retains native adoption custody.
pub use super::*;
pub use std::num::NonZeroUsize;
pub use std::time::Duration;
pub use worth_query_host::facade::admission::authenticated_principal::*;
pub use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryPrincipalResolutionDenialKind, WorthQueryPrincipalResolutionRefusalKind,
};

worth_query_decl::facade::worth_query_authentication_method!(CertificationMethod);
worth_query_decl::facade::worth_query_authentication_audience!(DocumentRetentionHostAudience);
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
            let now = worth_execution::time::system_time_sampler()
                .system_time()
                .unwrap();
            WorthQueryValidatedExternalPrincipal::new(
                worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::operator_identity(),
                WorthQueryAuthenticationAudience::of(DocumentRetentionHostAudience),
                WorthQueryAuthenticationMethod::of(CertificationMethod),
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
    time: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionTimeController,
) -> WorthQueryAuthenticatedExternalPrincipal<
    worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
>{
    let adapter = admit_authentication_adapter(
        host.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(DocumentRetentionHostAudience),
            WorthQueryAuthenticationMethod::of(CertificationMethod),
            time.clock(),
        ),
        ExpiringOperator,
    )
    .unwrap();
    let principal =
        worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::block_on(
            adapter.authenticate(Duration::from_millis(50), scope),
        )
        .unwrap();
    time.advance(Duration::from_secs(1)).unwrap();
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
