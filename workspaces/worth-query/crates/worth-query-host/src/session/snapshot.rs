//! Same-basis reads: a session snapshot pins one committed basis of the
//! session's branch, so every query it runs reads that basis however many
//! commits land meanwhile. Dropping the snapshot releases the pin.
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibility, WorthQueryAbilityKey, WorthQueryEligibilityQuestion,
};
use worth_query_publication::facade::{
    application_entry::{
        WorthQueryApplicationReadObservation, WorthQueryApplicationReadObservationDenial,
        WorthQueryApplicationRequestQueryDenial, WorthQueryApplicationRetainedRequest,
    },
    domain_computation::WorthQueryPublishedApplicationResult,
};
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductBranchIdentity};

use super::eligibility::{WorthQueryEligibilityDenial, WorthQueryEligibilityRoster};
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
    pub(crate) session: &'session WorthQueryHostSession<'home, Schema, Program>,
    pub(crate) observation: WorthQueryApplicationReadObservation,
}

impl<'home, Schema: ApplicationSchema, Program> WorthQueryHostSession<'home, Schema, Program> {
    /// Pins the session branch's current committed basis for same-basis reads.
    pub fn snapshot(
        &self,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionSnapshotDenial,
    > {
        let observation = self.retain_read_observation()?;
        Ok(WorthQuerySessionSnapshot {
            session: self,
            observation,
        })
    }

    pub(crate) fn retain_read_observation(
        &self,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQuerySessionSnapshotDenial> {
        let scope = self.fresh_scope();
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        self.with_funded_advancement(&scope, |phase| request.retain_read(&phase))
            .map_err(WorthQueryApplicationReadObservationDenial::ExecutionRequest)?
            .map_err(WorthQueryApplicationReadObservationDenial::ProductSelection)
    }
}

impl<'home, Schema: ApplicationSchema, Program>
    WorthQuerySessionSnapshot<'_, 'home, Schema, Program>
{
    /// Lends a request retained at the pinned basis, under the session's
    /// current proof and a fresh request scope, for reads beyond typed
    /// queries: workflow definition discovery, instance and history pages and
    /// original-start reads all observe exactly [`Self::selected_commit`].
    /// Each read still admits the principal's current disclosure; nothing is
    /// written and the pin stays held.
    pub fn with_request<R>(
        &self,
        body: impl for<'request> FnOnce(
            &WorthQueryApplicationRetainedRequest<'home, 'request, 'request, Schema>,
        ) -> R,
    ) -> R {
        let scope = self.session.fresh_scope();
        let principal = self.session.call_principal();
        let request = self.session.request_as(&principal, &scope);
        body(&request.at(&self.observation))
    }

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

    /// [`WorthQueryHostSession::eligibility`] as of [`Self::selected_commit`]:
    /// the policy decision for this principal at the pinned basis, beside rows
    /// read there. The asker is still admitted at the head, so a pin never
    /// widens who may ask. Fact-free; an action still admits at the head.
    pub fn eligibility<Context, Ability, Field>(
        &self,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<WorthQueryAbilityEligibility, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        super::eligibility::observe_asker(self.session, Some(&self.observation), question)
    }

    /// [`WorthQueryHostSession::eligible_among`] with verdicts as of
    /// [`Self::selected_commit`]. The asker's admission and `disclosure` are
    /// decided at the head: a revoked observer cannot read an older roster.
    pub fn eligible_among<Context, Ability, Disclosure, Field>(
        &self,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: ApplicationAbilityRef<Schema, Disclosure, Field::Entity>,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<WorthQueryEligibilityRoster, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let disclosure = WorthQueryAbilityKey::of(disclosure);
        super::eligibility::observe_roster(
            self.session,
            Some(&self.observation),
            question,
            disclosure,
            candidates,
        )
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
