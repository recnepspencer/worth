//! Fresh owner admission for live and durable recorded history coordinates.
use super::*;

impl<'application, 'principal, 'scope, Schema: ApplicationSchema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
{
    /// Selects the exact committed product occurrence identified by a receipt.
    pub fn at_commit(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        commit: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.at_selected_commit(
            phase,
            commit.committed_product_publication().composite_commit(),
            maximum_history_work,
        )
    }

    /// Select original durable receipt data through this live owner's history.
    /// The Home-scoped coordinate grants no branch admission or publication authority.
    pub fn at_recorded_commit(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        commit: worth_query_execution::facade::primary_graph::DurableCompositeCommitIdentity,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.select_commit_matching(phase, maximum_history_work, |selected| {
            selected.owner_identity().durable() == Some(commit.owner())
                && selected.ordinal() == commit.ordinal()
        })
    }

    /// Resolve original receipt data through this freshly admitted owner's history.
    /// Descriptive coordinates cannot create an observation or revive retired history.
    pub fn at_original_receipt(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        receipt: &worth_query_execution::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        use worth_query_execution::facade::primary_graph::RecordedProductBranchReference;
        self.select_commit_matching(phase, maximum_history_work, |selected| {
            match receipt.new_product_head() {
                RecordedProductBranchReference::Home(head) => {
                    selected.owner_identity().durable() == Some(head.branch().owner())
                        && selected.ordinal() == head.selected_commit().ordinal()
                }
                RecordedProductBranchReference::Memory(head) => {
                    selected.owner_identity().durable().is_none()
                        && selected.owner_identity().get() == head.owner()
                        && selected.ordinal() == head.selected_commit()
                }
            }
        })
    }

    /// Selects the exact product occurrence that issued an approved elevation.
    pub fn at_approved_elevation(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        approved: &worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.at_selected_commit(
            phase,
            approved.approval_product_publication().composite_commit(),
            maximum_history_work,
        )
    }

    fn at_selected_commit(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        selected_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        self.select_commit_matching(phase, maximum_history_work, |selected| {
            selected == selected_commit
        })
    }

    fn select_commit_matching(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        maximum_history_work: Option<std::num::NonZeroUsize>,
        matches: impl Fn(&worth_runtime_world::facade::CompositeCommitIdentity) -> bool,
    ) -> Result<
        WorthQueryApplicationRetainedRequest<'application, 'principal, 'scope, Schema>,
        WorthQueryApplicationHistorySelectionDenial,
    > {
        let history = self
            .application
            .branches()
            .history(phase, self.branch, maximum_history_work)
            .map_err(WorthQueryApplicationHistorySelectionDenial::ProductSelection)?;
        let entry = history
            .entries()
            .find(|entry| matches(entry.selected_commit()))
            .ok_or(WorthQueryApplicationHistorySelectionDenial::CommitUnavailable)?;
        let selected = history
            .select(phase, &entry)
            .map_err(WorthQueryApplicationHistorySelectionDenial::ProductSelection)?;
        Ok(WorthQueryApplicationRetainedRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
            observation: selected.retain_application_read(),
        })
    }
}
