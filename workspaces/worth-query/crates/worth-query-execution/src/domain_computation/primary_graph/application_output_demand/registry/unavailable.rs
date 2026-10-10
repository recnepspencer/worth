//! Close only an authentic, cleanup-free, prepublication domain refusal.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenialKind as Kind, WorthQueryPreparedRequiredOutputSource,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_unavailable_source(
        &self,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        interest: &WorthQueryOutputDemandInterest,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial> {
        if !Arc::ptr_eq(&self.state, &interest.owner.state)
            || !Arc::ptr_eq(&self.state, &prepared.owner.state)
        {
            return Err(refusal(Kind::ForeignDemand));
        }
        // Terminal rows can still own queued prerequisite/settlement cleanup.
        // Only the existing admitted cleanup owner may discharge those claims.
        self.drain_terminal_cleanup_admitted(owner, admission)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        let record = state
            .records
            .get(&interest.key)
            .ok_or_else(|| refusal(Kind::Closed))?;
        let DemandState::Failed(denial) = &record.state else {
            return Err(refusal(Kind::SchedulingDeferred));
        };
        let work = record
            .source_commits
            .len()
            .checked_add(24)
            .and_then(|n| n.checked_add(denial.subject().len()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| refusal(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(work)
            .map_err(|_| refusal(Kind::WorkBudgetExceeded))?;
        admission
            .admit_read_scratch(denial.subject().len() as u64)
            .map_err(|_| refusal(Kind::RetentionBudgetExceeded))?;
        if denial.kind() != Kind::ProducerDomainDenied
            || denial.recovery_posture() != crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
            || record.product_occurrence != prepared.product_occurrence
            || !record.source_commits.contains(&prepared.source_commit)
            || record.interests != 1 || record.required_interests != 1
            || record.framework_required_count != 0 || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued || record.held_successor.is_some()
            || record.successor_of.is_some() || record.performed_source.is_some()
            || !record.performed_obligations.is_empty() || !record.prerequisites.is_empty()
            || record.checkpoint_prerequisites.is_some() || !record.settlements.is_empty() {
            return Err(refusal(Kind::SchedulingDeferred));
        }
        let result = denial.clone();
        let custody = state
            .source_custody
            .get_mut(&prepared.source_commit)
            .ok_or_else(|| refusal(Kind::RetainedBasisUnavailable))?;
        if custody.retired.is_some()
            || custody.completed
            || custody.token_count != 1
            || custody.occurrence != prepared.product_occurrence
            || !matches!(
                custody.root_kind,
                source_custody::PreparedOutputRootKind::Required(_)
            )
            || custody.bound_sources.as_ref().is_none_or(|sources| {
                sources.len() != 1
                    || sources[0].identity != interest.key.source
                    || !custody.consumed_sources.contains(&sources[0].identity)
            })
        {
            return Err(refusal(Kind::ForeignSource));
        }
        // This is terminal unavailability, never a Ready output. No change,
        // candidate, receipt or output authority is synthesized here.
        custody.completed = true;
        let source = custody.source.take();
        let discovery = custody.discovery.take();
        drop(state);
        drop((source, discovery));
        Ok(result)
    }
}
fn refusal(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        kind,
        "required output has no cleanup-free terminal domain refusal",
    )
}
