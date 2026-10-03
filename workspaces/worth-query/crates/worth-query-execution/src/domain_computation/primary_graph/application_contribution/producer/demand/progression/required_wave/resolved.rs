//! One completed required successor may release its exact predecessor edge.

use super::*;
use crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// A single old consumed identity was joined to a performed successor whose
/// exact Ready was already certified Current on this selected wave.
pub(in crate::domain_computation::primary_graph) struct MatchedRequiredPredecessor<'a> {
    old_identity: Arc<RecordedSettlementIdentity>,
    selected: &'a PositionedRelationalSnapshot,
    _current: &'a Arc<WorthQueryOutputDemandSettlement>,
}

impl MatchedRequiredPredecessor<'_> {
    pub(in crate::domain_computation::primary_graph) fn old_identity(
        &self,
    ) -> &RecordedSettlementIdentity {
        self.old_identity.as_ref()
    }

    /// The cutoff constructs its own positioned view of the issued snapshot.
    /// Join its coordinates to this invocation's certified wave before using
    /// the old consumed identity to choose a fresh handler path.
    pub(in crate::domain_computation::primary_graph) fn matches_cutoff_root(
        &self,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        admission.charge_external_work(4)?;
        let branch_work = self
            .selected
            .branch_id()
            .0
            .len()
            .checked_add(selected.branch_id().0.len())
            .and_then(|work| {
                work.checked_add(2 * std::mem::size_of::<PositionedRelationalSnapshot>())
            })
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(branch_work)?;
        Ok(
            self.selected.runtime_instance_id() == selected.runtime_instance_id()
                && self.selected.branch_id() == selected.branch_id()
                && self.selected.root_id() == selected.root_id()
                && self.selected.version_id() == selected.version_id()
                && self.selected.commit_id() == selected.commit_id()
                && self.selected.position() == selected.position(),
        )
    }
}

/// This borrow exists only while the caller handles the Current result on its
/// selected wave. It cannot turn the predecessor into a Current settlement.
/// `progress` is the successor this wave refreshed, when it refreshed one; a
/// row that was already Current resolves no predecessor edge exactly.
pub(in crate::domain_computation::primary_graph) struct ResolvedRequiredPredecessor<'a, Schema>
where
    Schema: ApplicationSchema,
{
    progress: Option<&'a RequiredFreshProgress<Schema>>,
    successor_ready: &'a SelectedReadyReadmission,
    current: &'a Arc<WorthQueryOutputDemandSettlement>,
}

impl<'a, Schema> ResolvedRequiredPredecessor<'a, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn from_current(
        progress: Option<&'a RequiredFreshProgress<Schema>>,
        successor_ready: &'a SelectedReadyReadmission,
        current: &'a Arc<WorthQueryOutputDemandSettlement>,
    ) -> Self {
        Self {
            progress,
            successor_ready,
            current,
        }
    }

    /// Match an actor-selected old consumed identity only to the real
    /// predecessor of the successor that this wave already proved Current.
    pub(in crate::domain_computation::primary_graph) fn match_pending<'matched>(
        &'matched self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        pending: &SelectedPendingConsumedOutput<'_>,
        positioned: &'matched PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<MatchedRequiredPredecessor<'matched>>, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(8)
            .map_err(|_| work_denial())?;
        let Some(progress) = self.progress else {
            return Ok(None);
        };
        let progress_name = progress.producer_identity();
        let comparison_work = progress_name
            .len()
            .checked_mul(2)
            .and_then(|work| work.checked_add(self.current.producer_identity().len()))
            .and_then(|work| work.checked_add(progress.predecessor().producer_identity().len()))
            .and_then(|work| work.checked_add(2))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(comparison_work)
            .map_err(|_| work_denial())?;
        if !std::ptr::eq(pending.selected_root(), positioned)
            || self.current.producer_identity() != progress_name
            || progress.predecessor().producer_identity() != progress_name
        {
            return Ok(None);
        }
        if !self
            .successor_ready
            .matches_interest(progress.interest(), admission)?
        {
            return Ok(None);
        }
        let Some(live_successor) = runtime
            .output_demands
            .interest_ready_readmission(progress.interest(), admission)?
        else {
            return Ok(None);
        };
        if !live_successor.same_ready_cell(self.successor_ready, admission)? {
            return Ok(None);
        }
        preclaim_required_settlement_arguments(admission)?;
        let predecessor = runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .resolve_required_settlement(
                runtime.runtime.authority_identity().as_u64(),
                &runtime.installed_schema.binding_identity(),
                progress.predecessor().completion(),
                admission,
            );
        let predecessor = match predecessor {
            Ok(Some(predecessor)) => predecessor,
            Ok(None) | Err(FullVerificationReason::ForeignSource) => return Ok(None),
            Err(FullVerificationReason::MarkingAdmissionDenied(stop)) => {
                return Err(admission_denial(stop));
            }
            Err(_) => return Ok(None),
        };
        let identity_work = std::mem::size_of::<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >()
        .checked_mul(2)
        .and_then(|work| work.checked_add(std::mem::size_of::<MatchedRequiredPredecessor<'_>>()))
        .and_then(|work| work.checked_add(4))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(work_denial)?;
        admission
            .charge_external_work(identity_work)
            .map_err(|_| work_denial())?;
        if predecessor.recorded_identity() != pending.identity() {
            return Ok(None);
        }
        let old_identity = pending
            .retain_identity(admission)
            .map_err(admission_denial)?;
        Ok(Some(MatchedRequiredPredecessor {
            old_identity,
            selected: positioned,
            _current: self.current,
        }))
    }

    /// Whether `pending` names an older settlement of the row this wave just
    /// certified Current. No exact predecessor joins that edge, so the
    /// consumer refreshes in full against the Current row: holding it would
    /// wait on an upstream with nothing left to refresh.
    pub(in crate::domain_computation::primary_graph) fn names_current_upstream(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        pending: &SelectedPendingConsumedOutput<'_>,
        positioned: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let crate::domain_computation::primary_graph::application_output_demand::PendingUpstream::Ready(
            upstream,
        ) = runtime
            .output_demands
            .pending_exact_ready_readmission(pending, positioned, admission)?
        else {
            return Ok(false);
        };
        upstream.same_ready_cell(self.successor_ready, admission)
    }
}
