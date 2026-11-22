//! Retain actual applied READ, acquisition and provider custody after projection.
use crate::domain_computation::primary_graph::application_attempt::fact::source_read_authorization::applied::SourceReadAuthorizationAppliedExt;
mod publication;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::{
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};

pub struct AppliedManagedSourceSupport {
    source: Arc<WorthQueryAppliedSourceRead>,
    association: WorthQueryInstalledManagedIndexParent,
    acquisition: super::super::source::projection_support::AppliedSourceAcquisitionSupport,
    authorization: Fact,
    parameters: [u8; 32],
    published: std::sync::OnceLock<
        Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    >,
    _material: Payload,
    _coverage: Option<Arc<RetainedCoverage>>,
    _allocation: Arc<SourceFactsAllocation>,
}
impl AppliedManagedSelectionLease<'_, '_, '_> {
    /// The original provider capacity proof is consulted before its borrow ends;
    /// the genuine Source terminal and complete physical facts stay owned.
    pub fn retain_support(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<AppliedManagedSourceSupport>, Denial> {
        self.proof.consult().map_err(Denial::Session)?;
        SourceFactsAllocation::work(execution, 1)?;
        Ok(self.support)
    }
}
impl AppliedManagedSourceSupport {
    pub fn retain(
        execution: &ExecutionRequest<'_, '_>,
        admitted: source::AdmittedSource,
        association: WorthQueryInstalledManagedIndexParent,
        coverage: Option<Arc<RetainedCoverage>>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Arc<Self>, Denial> {
        if !admitted
            .authorization
            .validates_applied_terminal(&admitted.read)
        {
            return Err(Denial::ForeignAssociation);
        }
        allocation.claim_arc::<Self>(execution)?;
        Ok(Arc::new(Self {
            source: admitted.read,
            association,
            acquisition: admitted.support,
            authorization: Fact::SourceReadAuthorization {
                observation: admitted.authorization,
            },
            parameters: admitted.parameters,
            published: std::sync::OnceLock::new(),
            _material: admitted.material,
            _coverage: coverage,
            _allocation: Arc::clone(allocation),
        }))
    }
    pub fn authorization(&self) -> &Arc<SourceReadAuthorization> {
        let Fact::SourceReadAuthorization { observation } = &self.authorization else {
            unreachable!("Source acquisition retains its authorization")
        };
        observation
    }
    pub fn commitment(&self) -> [u8; 32] {
        self.acquisition.commitment()
    }
    pub fn published(
        &self,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    > {
        self.published.get()
    }

    pub fn candidate(&self) -> &RelationalAppliedCandidateReadIdentity {
        self.source.candidate()
    }
    pub fn facts(&self) -> &[Fact] {
        self.acquisition.facts()
    }
    pub fn authorization_fact(&self) -> &Fact {
        &self.authorization
    }
    /// Copy already acquired support into the existing mutation Source fact
    /// owner. These are captured coordinates, never a reacquisition of rows.
    pub fn append_facts(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &RelationalAppliedCandidateReadIdentity,
        facts: &mut Vec<Fact>,
    ) -> Result<(), worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        if self.candidate().qualification() != candidate.qualification()
            || self.acquisition.candidate().qualification() != candidate.qualification()
        {
            return Err(Cause::ExpectedIdentitiesNotCanonical);
        }
        let count = self
            .facts()
            .len()
            .checked_add(1)
            .ok_or(Cause::ChargedBytesOverflow)?;
        self._allocation.claim_slice::<Fact>(execution, count)?;
        facts
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        for fact in self
            .facts()
            .iter()
            .chain(std::iter::once(self.authorization_fact()))
        {
            SourceFactsAllocation::work(execution, 1)?;
            facts.push(fact.clone());
        }
        Ok(())
    }
    pub fn changed_fact<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        after_own: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        complete: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&'a Fact>, WorthQueryApplicationAttemptDenial> {
        if !after_own
            .candidate_identity()
            .includes_assembly_prefix(self.candidate())
            || !complete
                .candidate_identity()
                .includes_assembly_prefix(after_own.candidate_identity())
            || self.source.obligation() != self.association.access_contract().source_obligation()
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                "applied managed Source affinity",
            ));
        }
        for fact in self
            .facts()
            .iter()
            .chain(std::iter::once(self.authorization_fact()))
        {
            if !fact
                .support_equal_after_own_effect(execution, runtime, before, after_own, complete)?
            {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
}
