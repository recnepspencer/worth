//! Controllable local desktop principal source for temporal catch-up tests.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        admit_authentication_adapter, WorthQueryAdmissionClock,
        WorthQueryAdmittedAuthenticationAdapter, WorthQueryAuthenticationAdapterAdmission,
        WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
        WorthQueryCancellationSource, WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
    },
    domain, primary_graph,
};

use super::super::schema::DesktopReminderSchema;

#[derive(Clone)]
pub struct AuthControl {
    valid: Arc<AtomicBool>,
}
impl AuthControl {
    pub fn valid() -> Self {
        Self {
            valid: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn set_valid(&self, valid: bool) {
        self.valid.store(valid, Ordering::SeqCst);
    }
    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::SeqCst)
    }
}

pub fn admit_local_adapter(
    schema: &domain::WorthQueryInstalledApplicationSchema<DesktopReminderSchema>,
) -> WorthQueryAdmittedAuthenticationAdapter<DesktopReminderSchema, WorthQueryLocalProcessPrincipal>
{
    let clock = WorthQueryAdmissionClock::system();
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("desktop").unwrap(),
            WorthQueryAuthenticationMethod::new("local").unwrap(),
            clock,
        ),
        WorthQueryLocalProcessPrincipal::new(
            "worth.query.court.desktop-reminder.local-principal.v1",
            WorthQueryExternalPrincipalIdentity::new("https://local.desktop", "daymark-user")
                .unwrap(),
            WorthQueryAuthenticationAudience::new("desktop").unwrap(),
            WorthQueryAuthenticationMethod::new("local").unwrap(),
            Duration::from_secs(3600),
        )
        .unwrap(),
    )
    .unwrap()
}

pub fn fresh_request_scope() -> WorthQueryRequestScope {
    let clock = WorthQueryAdmissionClock::system();
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(clock.now() + Duration::from_secs(60)),
        cancellation.token(),
        clock,
    )
}

pub struct DesktopPrincipalSource {
    pub adapter: Arc<
        WorthQueryAdmittedAuthenticationAdapter<
            DesktopReminderSchema,
            WorthQueryLocalProcessPrincipal,
        >,
    >,
    pub auth: AuthControl,
}
impl primary_graph::WorthQueryTemporalPrincipalSource<DesktopReminderSchema>
    for DesktopPrincipalSource
{
    const SEMANTIC_IDENTITY: &'static str =
        "worth.query.court.desktop-reminder.principal-source.v1";

    fn admit(
        &self,
    ) -> Result<
        primary_graph::WorthQueryTemporalPrincipalAdmission<DesktopReminderSchema>,
        primary_graph::WorthQueryTemporalPrincipalFailure,
    > {
        if !self.auth.is_valid() {
            return Err(primary_graph::WorthQueryTemporalPrincipalFailure::new(
                primary_graph::WorthQueryTemporalPrincipalFailureKind::AdmissionRejected,
                "desktop principal expired",
            ));
        }
        let scope = fresh_request_scope();
        let external = self.adapter.authenticate_local(&scope).map_err(|denial| {
            primary_graph::WorthQueryTemporalPrincipalFailure::new(
                primary_graph::WorthQueryTemporalPrincipalFailureKind::AdmissionRejected,
                format!("{denial:?}"),
            )
        })?;
        Ok(primary_graph::WorthQueryTemporalPrincipalAdmission::new(
            external, scope,
        ))
    }
}
