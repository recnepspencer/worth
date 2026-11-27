//! The desktop surface of an ordinary Home, on its workflow Home.
//!
//! Each method is the same call on the paired [`WorthQueryApplicationHome`],
//! with the same signature, so code moves between the two Homes unchanged.
//! A session from [`WorthQueryWorkflowApplicationHome::session`] reaches the
//! same runtime as [`WorthQueryWorkflowApplicationHome::vocabulary`], so its
//! requests and snapshots accept that vocabulary for every workflow read and
//! lifecycle call.
use super::*;
use std::borrow::Borrow;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryLocalProcessPrincipal,
};
use worth_query_execution::facade::{
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime, product::WorthQueryProductBranch,
};

use crate::application_home::WorthQueryHomeOpenDisposition;
use crate::background::commit_watch::{
    WorthQueryCommitWatchDenial, WorthQueryCommitWatchGuard, WorthQueryCommittedBasisVersion,
};
use crate::session::{WorthQueryHostSession, WorthQueryLocalSessionDenial};

/// Sessions, commit observation and identity of the paired Home.
///
/// ```
/// use worth_query_host::facade::{
///     admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal,
///     declaration::{application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema},
///     WorthQueryWorkflowApplicationHome,
/// };
/// fn desktop<S, Spec, P>(
///     home: &WorthQueryWorkflowApplicationHome<S, Spec, P>,
///     reviewer: &WorthQueryAuthenticatedExternalPrincipal<S>,
///     repaint: impl Fn() + Send + Sync + 'static,
/// ) where S: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = S> {
///     let _watch = home.on_commit(repaint);
///     let session = home.session(reviewer);
///     let scope = session.fresh_scope();
///     // Workflow calls take the Home's vocabulary on the session's request:
///     let _request = session.request(&scope);
///     let _vocabulary = home.vocabulary();
/// }
/// ```
impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// [`WorthQueryApplicationHome::session`]: a reusable authenticated
    /// session on this Home's current active branch.
    pub fn session<'home>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    ) -> WorthQueryHostSession<'home, Schema, Program> {
        self.home.session(principal)
    }

    /// [`WorthQueryApplicationHome::local_session`]: a session acting as the
    /// local operating-system process, renewing its proof before each lapse.
    pub fn local_session(
        &self,
        local: WorthQueryLocalProcessPrincipal,
        clock: &WorthQueryAdmissionClock,
    ) -> Result<WorthQueryHostSession<'_, Schema, Program>, WorthQueryLocalSessionDenial> {
        self.home.local_session(local, clock)
    }

    /// [`WorthQueryApplicationHome::on_commit`]: calls `on_commit` after every
    /// commit to this Home, including workflow publication, start, proposal,
    /// approval and advance, until the guard drops or the Home closes.
    pub fn on_commit(
        &self,
        on_commit: impl Fn() + Send + Sync + 'static,
    ) -> Result<WorthQueryCommitWatchGuard, WorthQueryCommitWatchDenial> {
        self.home.on_commit(on_commit)
    }

    /// [`WorthQueryApplicationHome::committed_basis_version`].
    pub fn committed_basis_version(&self) -> WorthQueryCommittedBasisVersion {
        self.home.committed_basis_version()
    }

    /// [`WorthQueryApplicationHome::primary_graph`].
    pub fn primary_graph(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.home.primary_graph()
    }

    /// [`WorthQueryApplicationHome::active_branch`].
    pub fn active_branch(&self) -> WorthQueryProductBranch {
        self.home.active_branch()
    }

    /// [`WorthQueryApplicationHome::open_disposition`].
    pub const fn open_disposition(&self) -> WorthQueryHomeOpenDisposition {
        self.home.open_disposition()
    }

    /// [`WorthQueryApplicationHome::home_identity`].
    pub fn home_identity(&self) -> Option<worth_foundational::facade::HomeIdentity> {
        self.home.home_identity()
    }

    /// [`WorthQueryApplicationHome::home_incarnation`].
    pub fn home_incarnation(&self) -> Option<worth_foundational::facade::HomeIncarnation> {
        self.home.home_incarnation()
    }
}
