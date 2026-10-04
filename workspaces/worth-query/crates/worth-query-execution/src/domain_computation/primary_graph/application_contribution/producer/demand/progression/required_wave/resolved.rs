//! Completed required successors may release their exact predecessor edges.

use super::*;
use crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// Old consumed identities, each joined to the successor row whose exact
/// Ready was already certified Current on this selected wave, whether this
/// wave or an earlier advance refreshed it. A consumer of several outputs
/// carries one per resolved consumed edge.
pub(in crate::domain_computation::primary_graph) struct MatchedRequiredPredecessors<'a> {
    old_identities: Vec<Arc<RecordedSettlementIdentity>>,
    selected: &'a PositionedRelationalSnapshot,
}

impl<'a> MatchedRequiredPredecessors<'a> {
    /// Adds one matched edge to `matched`, starting the set on its first.
    pub(in crate::domain_computation::primary_graph) fn join(
        matched: &mut Option<Self>,
        old_identity: Arc<RecordedSettlementIdentity>,
        selected: &'a PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let item = std::mem::size_of::<Arc<RecordedSettlementIdentity>>();
        admission
            .charge_external_work(u64::try_from(item + 1).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let matched = matched.get_or_insert_with(|| Self {
            old_identities: Vec::new(),
            selected,
        });
        if matched.old_identities.len() == matched.old_identities.capacity() {
            let next = matched.old_identities.capacity().saturating_mul(2).max(1);
            admission
                .admit_read_scratch(
                    u64::try_from(next.saturating_mul(item)).map_err(|_| capacity_denial())?,
                )
                .map_err(admission_denial)?;
            matched
                .old_identities
                .try_reserve_exact(next - matched.old_identities.len())
                .map_err(|_| capacity_denial())?;
        }
        matched.old_identities.push(old_identity);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn old_identities(
        &self,
    ) -> &[Arc<RecordedSettlementIdentity>] {
        &self.old_identities
    }

    /// The cutoff constructs its own positioned view of the issued snapshot.
    /// Join its coordinates to this invocation's certified wave before using
    /// the old consumed identities to choose a fresh handler path.
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

/// The rows this wave certified Current while resolving one dependent's
/// consumed edges. A dependent of several outputs is readmitted once every
/// pending edge it names has resolved here. A moved wave starts over.
#[derive(Default)]
pub(super) struct ResolvedOnWave {
    entries: Vec<(
        SelectedReadyReadmission,
        Arc<WorthQueryOutputDemandSettlement>,
    )>,
}

impl ResolvedOnWave {
    pub(super) fn push(
        &mut self,
        ready: SelectedReadyReadmission,
        current: Arc<WorthQueryOutputDemandSettlement>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let item = std::mem::size_of::<(
            SelectedReadyReadmission,
            Arc<WorthQueryOutputDemandSettlement>,
        )>();
        admission
            .charge_external_work(u64::try_from(item + 1).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        if self.entries.len() == self.entries.capacity() {
            let next = self.entries.capacity().saturating_mul(2).max(1);
            admission
                .admit_read_scratch(
                    u64::try_from(next.saturating_mul(item)).map_err(|_| capacity_denial())?,
                )
                .map_err(admission_denial)?;
            self.entries
                .try_reserve_exact(next - self.entries.len())
                .map_err(|_| capacity_denial())?;
        }
        self.entries.push((ready, current));
        Ok(())
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    /// Whether `upstream` already resolved on this wave: asking for it again
    /// cannot progress the dependent.
    pub(super) fn contains(&self, upstream: &SelectedReadyReadmission) -> bool {
        self.entries
            .iter()
            .any(|(prior, _)| prior.completion().same_cell(upstream.completion()))
    }

    pub(super) fn view<'a, Schema: ApplicationSchema>(
        &'a self,
        custody: &'a [RequiredFreshProgress<Schema>],
    ) -> Option<ResolvedRequiredPredecessors<'a, Schema>> {
        (!self.entries.is_empty()).then_some(ResolvedRequiredPredecessors {
            resolved: &self.entries,
            custody,
        })
    }
}

/// This borrow exists only while the caller handles the Current results on
/// its selected wave. It cannot turn a predecessor into a Current settlement.
/// `custody` holds the successors this wave refreshed; a row that was
/// already Current has none there and resolves no predecessor edge exactly.
pub(in crate::domain_computation::primary_graph) struct ResolvedRequiredPredecessors<'a, Schema>
where
    Schema: ApplicationSchema,
{
    resolved: &'a [(
        SelectedReadyReadmission,
        Arc<WorthQueryOutputDemandSettlement>,
    )],
    custody: &'a [RequiredFreshProgress<Schema>],
}

impl<Schema> ResolvedRequiredPredecessors<'_, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Match an actor-selected old consumed identity only to the real
    /// predecessor of a successor that this wave already proved Current.
    pub(in crate::domain_computation::primary_graph) fn match_pending(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        pending: &SelectedPendingConsumedOutput<'_>,
        positioned: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<RecordedSettlementIdentity>>, WorthQueryOutputDemandDenial> {
        for (successor_ready, current) in self.resolved {
            let mut progress = None;
            for entry in self.custody.iter().rev() {
                if successor_ready.matches_interest(entry.interest(), admission)? {
                    progress = Some(entry);
                    break;
                }
            }
            let Some(progress) = progress else {
                continue;
            };
            if let Some(old) = match_one(
                runtime,
                progress,
                successor_ready,
                current,
                pending,
                positioned,
                admission,
            )? {
                return Ok(Some(old));
            }
        }
        Ok(None)
    }

    /// `pending`'s old identity when its live upstream row is one this wave
    /// certified Current, though no refresh of this wave replaced it: an
    /// earlier advance already did. The consumer refreshes against that
    /// Current row; holding it would wait on an upstream with nothing left
    /// to refresh.
    pub(in crate::domain_computation::primary_graph) fn match_current_upstream(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        pending: &SelectedPendingConsumedOutput<'_>,
        positioned: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<RecordedSettlementIdentity>>, WorthQueryOutputDemandDenial> {
        let crate::domain_computation::primary_graph::application_output_demand::PendingUpstream::Ready(
            upstream,
        ) = runtime
            .output_demands
            .pending_exact_ready_readmission(pending, positioned, admission)?
        else {
            return Ok(None);
        };
        for (successor_ready, _) in self.resolved {
            if upstream.same_ready_cell(successor_ready, admission)? {
                return pending
                    .retain_identity(admission)
                    .map(Some)
                    .map_err(admission_denial);
            }
        }
        Ok(None)
    }
}

/// One resolved successor's exact predecessor, when it is `pending`.
fn match_one<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    progress: &RequiredFreshProgress<Schema>,
    successor_ready: &SelectedReadyReadmission,
    current: &WorthQueryOutputDemandSettlement,
    pending: &SelectedPendingConsumedOutput<'_>,
    positioned: &PositionedRelationalSnapshot,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<Arc<RecordedSettlementIdentity>>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    admission
        .charge_external_work(8)
        .map_err(|_| work_denial())?;
    let progress_name = progress.producer_identity();
    let comparison_work = progress_name
        .len()
        .checked_mul(2)
        .and_then(|work| work.checked_add(current.producer_identity().len()))
        .and_then(|work| work.checked_add(progress.predecessor().producer_identity().len()))
        .and_then(|work| work.checked_add(2))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(comparison_work)
        .map_err(|_| work_denial())?;
    if !std::ptr::eq(pending.selected_root(), positioned)
        || current.producer_identity() != progress_name
        || progress.predecessor().producer_identity() != progress_name
    {
        return Ok(None);
    }
    let Some(live_successor) = runtime
        .output_demands
        .interest_ready_readmission(progress.interest(), admission)?
    else {
        return Ok(None);
    };
    if !live_successor.same_ready_cell(successor_ready, admission)? {
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
    let identity_work = std::mem::size_of::<RecordedSettlementIdentity>()
        .checked_mul(2)
        .and_then(|work| work.checked_add(4))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(identity_work)
        .map_err(|_| work_denial())?;
    if predecessor.recorded_identity() != pending.identity() {
        return Ok(None);
    }
    pending
        .retain_identity(admission)
        .map(Some)
        .map_err(admission_denial)
}
