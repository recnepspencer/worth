//! Separate real authentication identities; native admission resolves their entities.
use super::*;

pub fn reviewer_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(OPERATOR_ISSUER, "document-retention-reviewer")
        .unwrap()
}
pub fn ungranted_reviewer_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        OPERATOR_ISSUER,
        "document-retention-ungranted-reviewer",
    )
    .unwrap()
}
struct ReviewAdapter;
impl WorthQueryAuthenticationAdapter for ReviewAdapter {
    type Credential = bool;
    fn configuration_identity(&self) -> &str {
        "document-retention-review-identities.v1"
    }
    fn validate<'a>(
        &'a self,
        granted: bool,
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                if granted {
                    reviewer_identity()
                } else {
                    ungranted_reviewer_identity()
                },
                audience(),
                method(),
                now,
                now + Duration::from_secs(3600),
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
fn authenticate<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    scope: &WorthQueryRequestScope,
    granted: bool,
) -> WorthQueryAuthenticatedExternalPrincipal<S> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method()),
        ReviewAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate(granted, scope))
        .expect("reviewer authenticates through the installed adapter")
}
pub fn authenticate_reviewer<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<S> {
    authenticate(schema, scope, true)
}
pub fn authenticate_ungranted_reviewer<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<S> {
    authenticate(schema, scope, false)
}
