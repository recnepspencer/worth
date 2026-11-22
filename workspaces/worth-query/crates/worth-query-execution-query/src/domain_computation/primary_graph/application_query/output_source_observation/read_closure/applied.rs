//! A successful applied query read stays unpositioned until its actual publication.
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use crate::domain_computation::primary_graph::application_attempt::fact::source_read_authorization::applied_query::SourceReadAuthorizationAppliedQueryExt;
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
pub struct AppliedSourceQueryRead {
    proof: WorthQuerySessionGraphReadProof,
    pub authorization: Arc<SourceReadAuthorization>,
    candidate: RelationalAppliedCandidateReadIdentity,
    state: std::sync::Mutex<AppliedReadState>,
}
enum AppliedReadState {
    Unpublished(Candidate),
    Published(PositionedRelationalSnapshot),
}
impl SourceQueryReadClosure {
    pub fn retain_applied<Schema: ApplicationSchema>(
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
                .application_attempt_operation_control_ref()
                .clone(),
            facts: FactPayload::from(Arc::<[Fact]>::from(facts)),
            authority: SourceQueryReadAuthority::Applied(AppliedSourceQueryRead {
                proof,
                authorization,
                candidate: pin.candidate_identity().clone(),
                state: std::sync::Mutex::new(AppliedReadState::Unpublished(pin.clone())),
            }),
        }))
    }
    pub fn applied_candidate(&self) -> Option<&RelationalAppliedCandidateReadIdentity> {
        match &self.authority {
            SourceQueryReadAuthority::Applied(applied) => Some(&applied.candidate),
            SourceQueryReadAuthority::Committed { .. } => None,
        }
    }
    pub fn is_unpublished_applied(&self) -> bool {
        matches!(&self.authority, SourceQueryReadAuthority::Applied(applied) if applied.is_unpublished())
    }
    pub fn admits_applied_read(
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
            && &applied.candidate == candidate
            && applied
                .authorization
                .admits_applied_query(&applied.proof, candidate)
    }
    pub fn seal_applied_publication(
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
        let proposal = applied.candidate.proposal();
        if published.runtime_instance_id() != proposal.runtime_instance_id()
            || published.version_id() != proposal.proposed_version_id()
        {
            return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        let mut state = applied.state.lock().unwrap();
        if let AppliedReadState::Published(prior) = &*state {
            if prior.root_id() != published.root_id() || prior.commit_id() != published.commit_id()
            {
                return Err(ManagedIndexSourcePreparationDenial::ForeignSource);
            }
        } else {
            // Publication retains the positioned proof, not its preparation image.
            // The pin owns cold source custody and must retire at this transition.
            *state = AppliedReadState::Published(published);
        }
        Ok(())
    }
    pub fn applied_support_equal(
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
            .includes_assembly_prefix(&applied.candidate)
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
    pub fn is_unpublished(&self) -> bool {
        matches!(
            &*self.state.lock().unwrap(),
            AppliedReadState::Unpublished(_)
        )
    }
    pub fn source_facts_at_acquisition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        facts: &FactPayload,
    ) -> Result<Option<bool>, crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        let state = self.state.lock().unwrap();
        let AppliedReadState::Unpublished(pin) = &*state else {
            return Ok(None);
        };
        if !self
            .authorization
            .admits_applied_query(&self.proof, &self.candidate)
        {
            return Ok(Some(false));
        }
        for fact in facts.iter() {
            if !fact.comparison_in_image(runtime, execution, before, Some(pin))? {
                return Ok(Some(false));
            }
        }
        Ok(Some(true))
    }
}
