//! Real authentication expires while the native trusted clock services progression.
pub use super::lifecycle::ExpiringOperator;
pub use super::*;
pub use std::collections::VecDeque;
pub use std::num::NonZeroUsize;
pub use std::sync::{Arc, Mutex};
pub use std::time::{Duration, Instant, SystemTime};
pub use worth_query_host::facade::admission::authenticated_principal::*;
pub use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationRefusalKind, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};
#[derive(Clone, Default)]
pub struct ExpiryClock(pub Arc<Mutex<VecDeque<Option<Instant>>>>);
impl WorthQueryRuntimeTimeSource for ExpiryClock {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        let wait = self.0.lock().unwrap().pop_front().flatten();
        if let Some(deadline) = wait {
            std::thread::sleep(
                deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
            );
        }
        Ok(SystemTime::now())
    }
}
impl ExpiryClock {
    pub fn expire_during_progression(&self, deadline: Instant) {
        // Admission samples first; the existing owner's currentness refresh samples
        // next. Only the native clock blocks; no authorization decision is replaced.
        *self.0.lock().unwrap() = VecDeque::from([None, Some(deadline)]);
    }
    pub fn assert_serviced(&self) {
        assert!(self.0.lock().unwrap().is_empty());
    }
}
pub fn authenticate_live_expiring(
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
            adapter.authenticate(Duration::from_secs(1), scope),
        )
        .unwrap();
    assert!(!principal.is_expired());
    principal
}
pub fn expired(cause: &WorthQueryProgramAdoptionAuthorizationDenial) {
    match cause {
        WorthQueryProgramAdoptionAuthorizationDenial::Authorization(denial) => assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication
            )
        ),
        other => panic!(
            "expiry after access admission must reach the real capability progression: {other:?}"
        ),
    }
}

pub fn programs_publication(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    prepared: worth_query_host::facade::application_entry::WorthQueryPreparedBranchAdoption,
    principal: &WorthQueryAuthenticatedExternalPrincipal<
        worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
    >,
    scope: &WorthQueryRequestScope,
) -> WorthQueryBranchAdoptionPublicationOutcome {
    host.runtime()
        .request(principal, scope)
        .programs()
        .publish(prepared)
}
