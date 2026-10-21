mod cancellation;
mod checkpoint_custody;
pub(super) use checkpoint_custody::CheckpointPrerequisiteClaims;
mod consumed_predecessors;
mod evidence_handoff;
mod prerequisite_capacity;
mod prerequisite_denials;
mod publication;
use prerequisite_capacity::validate_predecessors;
use prerequisite_denials::capacity_denial;
use prerequisite_denials::closed_denial;
use prerequisite_denials::coverage_denial;
use std::sync::Arc;

use super::required_context::RequiredOutputDemandContext;
use super::DemandState;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

/// Claims the exact previously settled inputs of one running output before its
/// product effect. The successful publication moves these claims into the
/// demand record; dropping the ticket reverses them.
pub(in crate::domain_computation::primary_graph) struct PreparedPrerequisiteClaims {
    context: RequiredOutputDemandContext,
    predecessors: Vec<Arc<WorthQueryOutputDemandKey>>,
    claim_batch: Option<super::claim_readthrough::ClaimBatch>,
    checkpoint_predecessors: Option<CheckpointPrerequisiteClaims>,
    reserved_identity: Option<Arc<RecordedSettlementIdentity>>,
    reserved_posting: Option<super::settlement_index::Posting>,
    reserved_cleanup: Option<Box<super::settlement_index::PendingVacancyCleanup>>,
    published: bool,
}

impl RequiredOutputDemandContext {
    pub(in crate::domain_computation::primary_graph) fn prepare_prerequisites<'a>(
        self,
        inputs: impl ExactSizeIterator<Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
        source_owner: &crate::domain_computation::primary_graph::SourceInvalidationOwner,
    ) -> Result<PreparedPrerequisiteClaims, WorthQueryOutputDemandDenial> {
        if self.work_membership().is_some() {
            // The performed path can require a local Full cue after World has
            // committed. Its token and queue link were admitted with the row.
        }
        let count = inputs.len();
        let mut predecessors = Vec::new();
        predecessors
            .try_reserve_exact(count)
            .map_err(|_| capacity_denial())?;

        let owner = WorthQueryOutputDemandRegistry::clone(self.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let downstream = self.key();
        let record = state.records.get(downstream).ok_or_else(closed_denial)?;
        if !matches!(record.state, DemandState::Running) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                "required output is no longer the running producer operation",
            ));
        }
        if record.prepared_prerequisite_claims != 0 {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::SchedulingDeferred,
                "output already has a prepared prerequisite publication",
            ));
        }
        let checkpoint_predecessors = consumed_predecessors::select(
            &state,
            downstream,
            inputs,
            source_owner,
            &mut predecessors,
        )?;
        predecessors.sort_unstable();
        predecessors.dedup();
        validate_predecessors(&state, &predecessors)?;
        let claim_batch = state.prepare_claim_readthrough(downstream, &predecessors)?;
        let mut downstream_record = state.records.get_mut(downstream).unwrap();
        let settlements = &mut downstream_record.settlements;
        settlements
            .try_reserve_exact(1)
            .map_err(|_| capacity_denial())?;
        downstream_record.finish();

        if let Some(batch) = &claim_batch {
            batch.install();
        }
        for upstream in &predecessors {
            let prepared = state
                .prepare_required_member(upstream)
                .expect("all prerequisite members were admitted together");
            state
                .records
                .get_mut(upstream.as_ref())
                .unwrap()
                .framework_required_count += 1;
            state.install_required_member(prepared);
        }
        state
            .records
            .get_mut(downstream)
            .unwrap()
            .prepared_prerequisite_claims += 1;

        drop(state);
        Ok(PreparedPrerequisiteClaims {
            context: self,
            predecessors,
            claim_batch,
            checkpoint_predecessors,
            reserved_identity: None,
            reserved_posting: None,
            reserved_cleanup: None,
            published: false,
        })
    }
}

impl PreparedPrerequisiteClaims {
    pub(in crate::domain_computation::primary_graph) fn work_membership(
        &self,
    ) -> Option<Arc<super::required_work::RequiredWorkMembership>> {
        self.context.work_membership()
    }
    pub(in crate::domain_computation::primary_graph) fn retains_reserved_identity(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> bool {
        self.reserved_identity
            .as_ref()
            .is_some_and(|held| Arc::ptr_eq(held, identity))
            && self.reserved_posting.is_some()
            && self.reserved_cleanup.is_some()
    }
}

#[cfg(test)]
pub(super) mod custody_note_steps;
