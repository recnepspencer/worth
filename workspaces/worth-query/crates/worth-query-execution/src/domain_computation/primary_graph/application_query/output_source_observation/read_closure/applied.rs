//! A successful applied query read stays unpositioned until its actual publication.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{SourceReadAuthorization, WorthQueryApplicationAttemptDenial as Denial},
    application_checkpoint::facts::FactPayload,
    WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof;
use worth_relational::facade::{
    runtime::{
        ManagedIndexSourcePreparationDenial, PositionedRelationalSnapshot,
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime,
    },
    snapshots::SnapshotHandle,
};
type Candidate = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
pub(super) struct AppliedSourceQueryRead {
    proof: WorthQuerySessionGraphReadProof,
    pub(super) authorization: Arc<SourceReadAuthorization>,
    pin: Candidate,
    pub(super) published: std::sync::OnceLock<PositionedRelationalSnapshot>,
}
impl SourceQueryReadClosure {
    pub(in crate::domain_computation::primary_graph::application_query) fn retain_applied<
        Schema: ApplicationSchema,
    >(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        proof: WorthQuerySessionGraphReadProof,
        authorization: Arc<SourceReadAuthorization>,
        pin: &Candidate,
        facts: Vec<Fact>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        if !authorization.admits_applied_query(&proof, pin.candidate_identity()) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let allocation =
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(
                execution,
            )
            .map_err(CompanionPreflightStop::Execution)?;
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        Ok(Arc::new(Self {
            control: application
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .clone(),
            facts: FactPayload::from(Arc::<[Fact]>::from(facts)),
            authority: SourceQueryReadAuthority::Applied(AppliedSourceQueryRead {
                proof,
                authorization,
                pin: pin.clone(),
                published: Default::default(),
            }),
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn applied_candidate(
        &self,
    ) -> Option<&RelationalAppliedCandidateReadIdentity> {
        match &self.authority {
            SourceQueryReadAuthority::Applied(applied) => Some(applied.pin.candidate_identity()),
            SourceQueryReadAuthority::Committed { .. } => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn is_unpublished_applied(&self) -> bool {
        matches!(&self.authority, SourceQueryReadAuthority::Applied(applied) if applied.published.get().is_none())
    }
    pub(in crate::domain_computation::primary_graph) fn admits_applied_read(
        &self,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
        candidate: &RelationalAppliedCandidateReadIdentity,
    ) -> bool {
        let SourceQueryReadAuthority::Applied(applied) = &self.authority else {
            return false;
        };
        applied.authorization.key().principal == principal
            && applied.authorization.key().scope == scope
            && applied.pin.candidate_identity() == candidate
            && applied
                .authorization
                .admits_applied_query(&applied.proof, candidate)
    }
    pub(in crate::domain_computation::primary_graph) fn seal_applied_publication(
        &self,
        runtime: &RelationalRuntime,
        after: &SnapshotHandle,
    ) -> Result<(), ManagedIndexSourcePreparationDenial> {
        let SourceQueryReadAuthority::Applied(applied) = &self.authority else {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        };
        let published = runtime
            .read_truth()
            .positioned_snapshot(after)
            .map_err(|_| ManagedIndexSourcePreparationDenial::ForeignSource)?;
        let proposal = applied.pin.candidate_identity().proposal();
        if published.runtime_instance_id() != proposal.runtime_instance_id()
            || published.version_id() != proposal.proposed_version_id()
        {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        if let Some(prior) = applied.published.get() {
            if prior.root_id() != published.root_id() || prior.commit_id() != published.commit_id()
            {
                return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
            }
        } else {
            applied
                .published
                .set(published)
                .map_err(|_| ManagedIndexSourcePreparationDenial::ForeignSource)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn applied_support_equal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        after_own: &Candidate,
        complete: &Candidate,
    ) -> Result<bool, Denial> {
        let SourceQueryReadAuthority::Applied(applied) = &self.authority else {
            return Ok(false);
        };
        if !after_own
            .candidate_identity()
            .includes_assembly_prefix(applied.pin.candidate_identity())
            || !complete
                .candidate_identity()
                .includes_assembly_prefix(after_own.candidate_identity())
            || !applied
                .authorization
                .supports_final_candidate(execution, runtime, before, complete)?
        {
            return Ok(false);
        }
        for fact in self.facts.iter() {
            if !fact
                .support_equal_after_own_effect(execution, runtime, before, after_own, complete)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
impl AppliedSourceQueryRead {
    pub(super) fn source_facts_at_acquisition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        facts: &FactPayload,
    ) -> Result<bool, crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        if !self
            .authorization
            .admits_applied_query(&self.proof, self.pin.candidate_identity())
        {
            return Ok(false);
        }
        for fact in facts.iter() {
            if !fact.comparison_in_image(runtime, execution, before, Some(&self.pin))? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
