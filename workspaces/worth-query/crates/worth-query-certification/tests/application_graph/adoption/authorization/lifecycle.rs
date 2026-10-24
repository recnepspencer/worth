//! Expired real authentication retains native adoption custody.
use super::*;
use std::num::NonZeroUsize;
use std::time::{Duration, SystemTime};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryPrincipalResolutionDenialKind, WorthQueryPrincipalResolutionRefusalKind,
};
pub(super) struct ExpiringOperator;
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
                crate::document_retention_model::operator_identity::operator_identity(),
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
fn expired_operator(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<
    crate::document_retention_model::schema::DocumentRetentionSchema,
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
    let principal = crate::document_retention_model::operator_identity::block_on(
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
fn expired(cause: WorthQueryProgramAdoptionAuthorizationDenial) {
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
#[test]
fn expired_authentication_retains_prepared_and_pending_owners_for_fresh_publication() {
    let host = publish_on_first_program();
    let a = host.current_world();
    let b = super::branch_set::fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&current, &scope)
        .on_branch(a)
        .programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[b], NonZeroUsize::new(1).unwrap())
        .unwrap();
    let mut pending = host
        .runtime()
        .request(&current, &scope)
        .on_branches(coverage, &[b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    let stale = expired_operator(&host, &scope);
    let prepared = match prepared.publish(host.runtime(), &stale, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            expired(cause);
            prepared
        }
        _ => panic!("expiry must retain the reservation"),
    };
    match pending
        .advance(host.runtime(), &stale, &scope)
        .err()
        .unwrap()
    {
        WorthQueryBranchSetAdoptionAdvanceDenial::Authorization { cause, .. } => expired(cause),
        other => panic!("expiry must retain pending native custody: {other:?}"),
    }
    assert_eq!(host.current_world(), a);
    assert_eq!(read_retention(host.runtime(), a), 7);
    assert!(matches!(
        prepared.publish(host.runtime(), &current, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert!(matches!(
        pending
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    assert!(pending.close().is_ok());
}
