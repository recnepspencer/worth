//! Same-basis reads: a session snapshot pins one committed basis of the
//! session's branch, so every query it runs reads that basis however many
//! commits land meanwhile. Dropping the snapshot releases the pin.
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_publication::facade::{
    application_entry::{
        WorthQueryApplicationReadObservation, WorthQueryApplicationReadObservationDenial,
        WorthQueryApplicationRequestQueryDenial,
    },
    domain_computation::WorthQueryPublishedApplicationResult,
};
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchIdentity};

use super::{
    QueryMarker, WorthQueryHostSession, WorthQueryPage, WorthQueryPageCursor,
    WorthQuerySessionQuery,
};

/// Why [`WorthQueryHostSession::snapshot`] could not pin a basis.
pub type WorthQuerySessionSnapshotDenial = WorthQueryApplicationReadObservationDenial;

/// One committed basis pinned for several typed queries.
///
/// From [`WorthQueryHostSession::snapshot`]. Each query still admits the
/// session's principal and grants and runs under the session's policy and
/// result limit; only the basis is shared. Two snapshots with equal
/// [`Self::selected_commit`] read identical data.
#[must_use = "a snapshot pins its basis until dropped"]
pub struct WorthQuerySessionSnapshot<'session, 'home, Schema: ApplicationSchema, Program> {
    session: &'session WorthQueryHostSession<'home, Schema, Program>,
    observation: WorthQueryApplicationReadObservation,
}

impl<'home, Schema: ApplicationSchema, Program> WorthQueryHostSession<'home, Schema, Program> {
    /// Pins the session branch's current committed basis for same-basis reads.
    pub fn snapshot(
        &self,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionSnapshotDenial,
    > {
        let scope = self.fresh_scope();
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let observation = self
            .with_funded_advancement(&scope, |phase| request.retain_read(&phase))
            .map_err(WorthQueryApplicationReadObservationDenial::ExecutionRequest)?
            .map_err(WorthQueryApplicationReadObservationDenial::ProductSelection)?;
        Ok(WorthQuerySessionSnapshot {
            session: self,
            observation,
        })
    }
}

impl<Schema: ApplicationSchema, Program> WorthQuerySessionSnapshot<'_, '_, Schema, Program> {
    /// The commit every query of this snapshot reads.
    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        self.observation.selected_commit()
    }

    /// The product branch the pinned commit belongs to.
    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        self.observation.branch_identity()
    }

    /// Runs `intent` at the pinned basis and returns its typed rows.
    pub fn query<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
    ) -> Result<Vec<Intent::Row>, WorthQueryApplicationRequestQueryDenial> {
        self.query_result(intent)
            .map(WorthQueryPublishedApplicationResult::into_rows)
    }

    /// [`WorthQueryHostSession::query_page`] at the pinned basis: reading every
    /// page through one snapshot walks one consistent collection, each row
    /// exactly once, however many commits land meanwhile.
    pub fn query_page<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
        width: std::num::NonZeroUsize,
        after: Option<&WorthQueryPageCursor<Intent>>,
    ) -> Result<WorthQueryPage<Intent::Row, Intent>, WorthQueryApplicationRequestQueryDenial> {
        super::page::read_page(self.session, Some(&self.observation), intent, width, after)
    }

    /// Runs `intent` at the pinned basis and keeps the full published result.
    pub fn query_result<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
    ) -> Result<
        WorthQueryPublishedApplicationResult<QueryMarker<Schema, Intent>, Intent::Row>,
        WorthQueryApplicationRequestQueryDenial,
    > {
        intent.execute_at_in_session(self.session, &self.observation)
    }
}
