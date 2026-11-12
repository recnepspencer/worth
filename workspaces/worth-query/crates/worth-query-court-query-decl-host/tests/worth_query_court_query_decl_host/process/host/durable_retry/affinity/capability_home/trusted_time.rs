//! Trusted Host time reaches actual capability admission on both Home recipes.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind as Denial,
    WorthQueryOperationAuthorizationRefusalKind as Refusal, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};

struct Clock {
    calls: Arc<AtomicUsize>,
    available: bool,
}
impl WorthQueryRuntimeTimeSource for Clock {
    fn admission_clock(
        &self,
    ) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock
    {
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()
    }
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.available {
            Ok(UNIX_EPOCH + Duration::from_secs(1_700_000_000))
        } else {
            Err(WorthQueryRuntimeTimeSourceDenial::Unavailable)
        }
    }
}

#[test]
fn directory_reopen_uses_the_new_trusted_clock_without_sampling_during_open() {
    let path = fixture::location();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let calls = Arc::new(AtomicUsize::new(0));
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request(
                WorthQueryApplicationHomeLocation::Directory(path.clone()),
                true,
            )
            .with_authorization_time_source(Clock {
                calls: calls.clone(),
                available: false,
            })
            .open_in_advancement(&phase)
            .unwrap_or_else(|refused| opening_failed("unavailable clock writer", &refused)),
        )
    })
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        admission(home.runtime()),
        Some(Denial::Refused(Refusal::TrustedTimeUnavailable))
    );
    assert!(calls.load(Ordering::SeqCst) > 0);
    drop(home);
    let calls = Arc::new(AtomicUsize::new(0));
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request(WorthQueryApplicationHomeLocation::Directory(path), false)
                .with_authorization_time_source(Clock {
                    calls: calls.clone(),
                    available: true,
                })
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("new clock reader", &refused)),
        )
    })
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(admission(home.runtime()), None);
    assert!(calls.load(Ordering::SeqCst) > 0);
}

#[test]
fn memory_home_uses_its_supplied_trusted_authorization_clock() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let calls = Arc::new(AtomicUsize::new(0));
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request(WorthQueryApplicationHomeLocation::MemoryOnly, true)
                .with_authorization_time_source(Clock {
                    calls: calls.clone(),
                    available: false,
                })
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("memory clock", &refused)),
        )
    })
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        admission(home.runtime()),
        Some(Denial::Refused(Refusal::TrustedTimeUnavailable))
    );
    assert!(calls.load(Ordering::SeqCst) > 0);
}

fn admission(
    home: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        HostIdentitySchema,
        CapabilityProgram,
    >,
) -> Option<Denial> {
    use identity::inspection_capability::{InspectAccountCapability, InspectAccountOperation};
    let (_, live) = scope();
    let app = home.runtime();
    let external = identity::inspection_authentication::authenticate(app.installed_schema(), &live);
    app.with_application_advancement(&live, |phase| {
        let selected = app.on_branch(app.current_world()).select(&phase).unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &phase,
                &app.installed_schema()
                    .principal_binding(identity::IdentityBinding::reference())
                    .unwrap(),
                &external,
                &live,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        selected
            .admit_capability_access(
                &phase,
                &principal,
                &app.installed_schema()
                    .capability(
                        InspectAccountCapability::reference(),
                        InspectAccountOperation::reference(),
                    )
                    .unwrap(),
                "account-001".to_owned(),
                &live,
            )
            .err()
            .map(|denial| denial.kind())
    })
    .unwrap()
}
