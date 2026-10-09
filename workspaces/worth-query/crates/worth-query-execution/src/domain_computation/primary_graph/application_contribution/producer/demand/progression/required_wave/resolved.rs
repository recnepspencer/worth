//! Completed required successors may release their exact predecessor edges.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::SelectedPendingConsumedOutput;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// An exact old edge joined to the Ready cell resolved on this selected wave.
pub(in crate::domain_computation::primary_graph) struct ResolvedConsumedPredecessor {
    old: Arc<RecordedSettlementIdentity>,
    successor: ReadyCompletion,
}
impl ResolvedConsumedPredecessor {
    fn retain(
        old: Arc<RecordedSettlementIdentity>,
        successor: &ReadyCompletion,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work((std::mem::size_of::<Self>() + 2) as u64)
            .map_err(admission_denial)?;
        Ok(Self {
            old,
            successor: successor.clone(),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (Arc<RecordedSettlementIdentity>, ReadyCompletion) {
        (self.old, self.successor)
    }
}

mod matched_predecessors;
pub(in crate::domain_computation::primary_graph) use matched_predecessors::{
    MatchedRequiredPredecessors, ReboundConsumedOutput,
};

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
    ) -> Result<Option<ResolvedConsumedPredecessor>, WorthQueryOutputDemandDenial> {
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
    ) -> Result<Option<ResolvedConsumedPredecessor>, WorthQueryOutputDemandDenial> {
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
                    .map_err(admission_denial)
                    .and_then(|old| {
                        ResolvedConsumedPredecessor::retain(
                            old,
                            successor_ready.completion(),
                            admission,
                        )
                        .map(Some)
                    });
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
) -> Result<Option<ResolvedConsumedPredecessor>, WorthQueryOutputDemandDenial>
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
    {
        return Ok(None);
    }
    // Required admission sealed the typed predecessor→successor relation.
    // Initial and Preserve may have different bindings; this match authenticates
    // the live successor cell and the exact consumed predecessor below.
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
    let predecessor = match predecessor.map_err(required_settlement_denial)? {
        Ok(Some(predecessor)) => predecessor,
        // No exact predecessor row: this edge matches nothing.
        Ok(None) | Err(_) => return Ok(None),
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
        .map_err(admission_denial)
        .and_then(|old| {
            ResolvedConsumedPredecessor::retain(old, successor_ready.completion(), admission)
                .map(Some)
        })
}
