//! Real authentication expires while the native trusted clock services progression.
pub use super::lifecycle::ExpiringOperator;
pub use super::*;
pub use std::collections::VecDeque;
pub use std::num::NonZeroUsize;
pub use std::sync::{Arc, Mutex};
pub use std::time::{Duration, Instant};
pub use worth_query_host::facade::admission::authenticated_principal::*;
pub use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind;
use worth_query_host::facade::primary_graph::WorthQueryRuntimeClock;
use worth_query_host::facade::runtime::time::{RawClockAdapter, SystemClockAdapter};
use worth_query_host::facade::runtime::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

worth_query_decl::facade::worth_query_authentication_method!(CertificationMethod);
worth_query_decl::facade::worth_query_authentication_audience!(DocumentRetentionHostAudience);
#[derive(Clone)]
pub struct ExpiryClock {
    scheduled: Arc<Mutex<VecDeque<Option<Instant>>>>,
    wall: Arc<SystemClockAdapter>,
    pub time: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionTimeController,
}
impl RawClockAdapter for ExpiryClock {
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        self.wall.read_monotonic_ticks()
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.wall.source_identity()
    }

    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        let wait = self.scheduled.lock().unwrap().pop_front().flatten();
        if let Some(deadline) = wait {
            let elapsed = deadline
                .checked_duration_since(self.time.clock().now())
                .expect("scheduled deadline is ahead of the held clock");
            self.time
                .advance(elapsed + Duration::from_nanos(1))
                .unwrap();
        }
        self.wall.read_wall_unix_nanos()
    }
}
impl ExpiryClock {
    pub fn expire_during_progression(&self, deadline: Instant) {
        // Admission samples first; the existing owner's currentness refresh samples
        // next. The time owner advances its real admission observer at that sample.
        *self.scheduled.lock().unwrap() = VecDeque::from([None, Some(deadline)]);
    }
    pub fn assert_serviced(&self) {
        assert!(self.scheduled.lock().unwrap().is_empty());
    }
}
pub fn authenticate_live_expiring(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<
    worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema,
>{
    let adapter = admit_authentication_adapter(
        host.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(DocumentRetentionHostAudience),
            WorthQueryAuthenticationMethod::of(CertificationMethod),
            scope.clock().clone(),
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

impl Default for ExpiryClock {
    fn default() -> Self {
        let time = WorthQueryAdmissionTimeController::new().unwrap();
        time.hold();
        Self {
            scheduled: Default::default(),
            wall: Arc::new(SystemClockAdapter::new(ClockSourceIdentity::new(0))),
            time,
        }
    }
}

impl From<ExpiryClock> for WorthQueryRuntimeClock {
    fn from(source: ExpiryClock) -> Self {
        let clock = source.time.clock();
        Self::from(Arc::new(source)).with_admission_clock(clock)
    }
}
