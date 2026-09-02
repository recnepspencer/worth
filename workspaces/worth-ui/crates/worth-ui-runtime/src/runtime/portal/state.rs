use std::collections::BTreeMap;

#[path = "state/commit.rs"]
mod commit;
#[path = "state/duplicate_request.rs"]
mod duplicate_request;
#[path = "state/mounted_projection.rs"]
mod mounted_projection;
#[path = "state/order_index.rs"]
mod order_index;
#[path = "state/preparation.rs"]
mod preparation;
#[path = "state/shutdown.rs"]
mod shutdown;

pub(crate) use shutdown::UiPortalShutdownReport;

#[cfg(test)]
pub(super) const fn duplicate_request_capacity_for_test() -> usize {
    duplicate_request::UI_PORTAL_CLOSED_REQUEST_CAPACITY
}

/// `records` holds only live portals: `Open`, `Visible`, or `Closing`. A portal
/// that reaches `Closed` leaves the live table and keeps only its bounded
/// duplicate-request row, so placement, dismissal, descendant, and command
/// routing work stays proportional to the currently active portals rather than
/// to every portal the session ever opened.
pub(crate) struct UiPortalRuntimeState {
    pub(super) policy: crate::declaration::UiPortalPolicy,
    pub(super) records: BTreeMap<super::UiPortalIdentity, UiPortalRecord>,
    closed_requests: duplicate_request::UiPortalClosedRequestWindow,
    admitted_requests: u64,
    idempotent_requests: u64,
    pub(super) revision: u64,
    pub(super) stack_ordinal_issuer: super::UiPortalStackOrdinalIssuer,
    pub(super) stack_order: order_index::UiPortalStackOrderIndex,
    last_closed: Option<super::UiPortalClosedInspectionRecord>,
}

pub(super) struct UiPortalRecord {
    pub(super) posture: super::UiPortalLifecyclePosture,
    pub(super) semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    last_request: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    dismissal: Option<super::UiPortalDismissalCause>,
    pub(super) placement: Option<super::UiCommittedPortalPlacement>,
    pub(super) stack_ordinal: super::UiPortalStackOrdinal,
    exit_retention: Option<super::UiPortalExitRetentionReceipt>,
}

impl UiPortalRuntimeState {
    pub(crate) fn new(persistence: crate::runtime::UiServiceStatePersistencePosture) -> Self {
        Self::new_with_policy(persistence, crate::declaration::UiPortalPolicy::dropdown())
    }

    pub(crate) fn new_with_policy(
        _persistence: crate::runtime::UiServiceStatePersistencePosture,
        policy: crate::declaration::UiPortalPolicy,
    ) -> Self {
        Self::new_with_policy_and_ordinal_issuer(
            _persistence,
            policy,
            super::UiPortalStackOrdinalIssuer::new(),
        )
    }

    pub(crate) fn new_with_policy_and_ordinal_issuer(
        _persistence: crate::runtime::UiServiceStatePersistencePosture,
        policy: crate::declaration::UiPortalPolicy,
        stack_ordinal_issuer: super::UiPortalStackOrdinalIssuer,
    ) -> Self {
        Self {
            policy,
            records: BTreeMap::new(),
            closed_requests: duplicate_request::UiPortalClosedRequestWindow::new(),
            admitted_requests: 0,
            idempotent_requests: 0,
            revision: 0,
            stack_ordinal_issuer,
            stack_order: order_index::UiPortalStackOrderIndex::new(),
            last_closed: None,
        }
    }

    pub(crate) fn apply_policy(&mut self, policy: crate::declaration::UiPortalPolicy) {
        self.policy = policy;
    }

    pub(crate) fn take_stack_ordinal_issuer(&mut self) -> super::UiPortalStackOrdinalIssuer {
        std::mem::replace(
            &mut self.stack_ordinal_issuer,
            super::UiPortalStackOrdinalIssuer::exhausted(),
        )
    }

    pub(crate) fn committed_presentation_for(
        &self,
        portal: super::UiPortalIdentity,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.records
            .get(&portal)?
            .placement
            .map(|placement| placement.prepared().presentation())
    }

    pub(crate) fn anchor_requires_dismissal(&self, portal: super::UiPortalIdentity) -> bool {
        self.records.get(&portal).is_some_and(|record| {
            matches!(
                record.posture,
                super::UiPortalLifecyclePosture::Open | super::UiPortalLifecyclePosture::Visible
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn posture(
        &self,
        portal: super::UiPortalIdentity,
    ) -> super::UiPortalLifecyclePosture {
        self.records
            .get(&portal)
            .map_or(super::UiPortalLifecyclePosture::Closed, |record| {
                record.posture
            })
    }

    #[cfg(test)]
    pub(crate) fn semantic_surface_for_test(
        &self,
        portal: super::UiPortalIdentity,
    ) -> Option<worth_ui_host_contract::UiSemanticSurfaceIdentity> {
        self.records
            .get(&portal)
            .map(|record| record.semantic_surface)
    }

    /// The live table holds exactly the active portals, so this is a length
    /// rather than a scan.
    pub(crate) fn active_count(&self) -> usize {
        self.records.len()
    }

    /// Graph nodes exposed by active Portal scopes are the Portal owner/anchor
    /// nodes in 3.15, not child content mounted inside the Portal. Bounded by
    /// the active portals because terminal portals leave the live table.
    pub(crate) fn active_portal_owner_graph_nodes(
        &self,
    ) -> impl Iterator<Item = crate::graph::UiGraphNodeIdentity> + '_ {
        self.records
            .keys()
            .map(|identity| identity.owner().graph_node())
    }

    pub(crate) fn posture_count(&self, posture: super::UiPortalLifecyclePosture) -> usize {
        self.records
            .values()
            .filter(|record| record.posture == posture)
            .count()
    }

    pub(crate) fn exit_retention_count(&self) -> usize {
        self.records
            .values()
            .filter(|record| record.exit_retention.is_some())
            .count()
    }

    pub(crate) const fn admitted_requests(&self) -> u64 {
        self.admitted_requests
    }

    pub(crate) const fn idempotent_requests(&self) -> u64 {
        self.idempotent_requests
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn last_closed(&self) -> Option<super::UiPortalClosedInspectionRecord> {
        self.last_closed
    }

    #[cfg(test)]
    pub(crate) fn force_next_stack_ordinal(&mut self, next: u64) {
        self.stack_ordinal_issuer.force_next(next);
    }

    #[cfg(test)]
    pub(crate) fn reconstruct_stack_order_for_test(&mut self) {
        self.stack_order = order_index::UiPortalStackOrderIndex::rebuild(&self.records);
    }

    /// Live portal records plus the bounded duplicate-request rows retained for
    /// terminally closed portals. Both must reach zero at shutdown.
    pub(crate) fn record_count(&self) -> usize {
        self.records.len() + self.closed_requests.len()
    }

    #[cfg(test)]
    pub(super) fn live_record_count(&self) -> usize {
        self.records.len()
    }

    pub(super) fn clear_closed_requests(&mut self) {
        self.closed_requests.clear();
    }
}
