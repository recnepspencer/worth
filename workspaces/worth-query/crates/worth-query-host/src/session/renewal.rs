//! Self-renewing session proofs: a session holding a principal source mints a
//! fresh proof before the current one lapses, so one long-lived desktop
//! session serves arbitrarily long uptimes without minting proofs per call.
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryAdmittedAuthenticationAdapter,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapterAdmissionDenial,
    WorthQueryAuthenticationDenial, WorthQueryCancellationSource, WorthQueryLocalProcessPrincipal,
    WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::WorthQueryHostSession;
use crate::application_home::WorthQueryApplicationHome;

/// A synchronous source of fresh sign-in proofs for one principal.
///
/// A session given a source with [`WorthQueryHostSession::with_principal_renewal`]
/// renews its proof once the current one enters the last quarter of its
/// lifetime. An admitted local-process adapter is a source.
pub trait WorthQuerySessionPrincipalSource<Schema>: Send + Sync + 'static {
    /// Mints a fresh proof under `scope`; every admission check still applies.
    fn authenticate(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, WorthQueryAuthenticationDenial>;
}

impl<Schema: 'static> WorthQuerySessionPrincipalSource<Schema>
    for WorthQueryAdmittedAuthenticationAdapter<Schema, WorthQueryLocalProcessPrincipal>
{
    fn authenticate(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, WorthQueryAuthenticationDenial>
    {
        self.authenticate_local(scope)
    }
}

/// Why [`WorthQueryApplicationHome::local_session`] could not sign in.
#[derive(Debug)]
pub enum WorthQueryLocalSessionDenial {
    Admission(WorthQueryAuthenticationAdapterAdmissionDenial),
    Authentication(WorthQueryAuthenticationDenial),
}

/// The latest proof of a renewing session, shared by every clone of it.
pub(crate) struct RenewingPrincipal<Schema> {
    source: Box<dyn WorthQuerySessionPrincipalSource<Schema>>,
    current: Mutex<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
}

impl<Schema: 'static> RenewingPrincipal<Schema> {
    /// The current proof, renewed first when it is in its last quarter. A
    /// refused renewal keeps the old proof, so the request reports
    /// `SessionExpired` once it lapses rather than a missing permission.
    pub(super) fn current(&self) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
        let mut current = self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if renewal_due(&current) {
            let scope = WorthQueryRequestScope::new(
                None,
                WorthQueryCancellationSource::new().token(),
                current.clock().clone(),
            );
            if let Ok(renewed) = self.source.authenticate(&scope) {
                *current = renewed;
            }
        }
        current.clone()
    }
}

fn renewal_due<Schema>(principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>) -> bool {
    let lifetime = principal
        .expires_at()
        .duration_since(principal.validated_at())
        .unwrap_or_default();
    principal
        .clock()
        .now()
        .checked_add(lifetime / 4)
        .is_none_or(|horizon| horizon >= principal.valid_until())
}

impl<'home, Schema: ApplicationSchema, Program> WorthQueryHostSession<'home, Schema, Program> {
    /// Renews this session's proof from `source` before it lapses. Clones of
    /// the session, and an owned session made from it, share the renewal.
    pub fn with_principal_renewal(
        mut self,
        source: impl WorthQuerySessionPrincipalSource<Schema>,
    ) -> Self {
        self.settings.renewal = Some(Arc::new(RenewingPrincipal {
            source: Box::new(source),
            current: Mutex::new(self.settings.principal.clone()),
        }));
        self
    }

    /// Whether this session renews its own proof.
    pub fn renews_principal(&self) -> bool {
        self.settings.renewal.is_some()
    }

    /// The proof this session's next operation acts under, renewed first when
    /// it is due. Pass it to Home calls that take a principal, such as
    /// `home.undo(..)`, `home.redo(..)` or `home.correction_history(..)`, so a
    /// renewing session needs no separate adapter past one proof lifetime.
    pub fn current_principal(&self) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
        self.call_principal().into_owned()
    }

    /// The proof one operation of this session acts under: the renewed proof
    /// for a renewing session, otherwise the session's fixed proof.
    pub(crate) fn call_principal(
        &self,
    ) -> Cow<'_, WorthQueryAuthenticatedExternalPrincipal<Schema>> {
        match &self.settings.renewal {
            Some(renewal) => Cow::Owned(renewal.current()),
            None => Cow::Borrowed(&self.settings.principal),
        }
    }
}

impl<Schema: ApplicationSchema + 'static, Program> WorthQueryApplicationHome<Schema, Program> {
    /// A session acting as the local operating-system process, renewing its
    /// proof before each lapse so it stays valid for the life of the process.
    ///
    /// `clock` is the admission clock this Home was opened with.
    pub fn local_session(
        &self,
        local: WorthQueryLocalProcessPrincipal,
        clock: &WorthQueryAdmissionClock,
    ) -> Result<WorthQueryHostSession<'_, Schema, Program>, WorthQueryLocalSessionDenial> {
        let adapter = local
            .admit(self.primary_graph().installed_schema(), clock.clone())
            .map_err(WorthQueryLocalSessionDenial::Admission)?;
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            clock.clone(),
        );
        let principal = adapter
            .authenticate_local(&scope)
            .map_err(WorthQueryLocalSessionDenial::Authentication)?;
        Ok(self.session(principal).with_principal_renewal(adapter))
    }
}
