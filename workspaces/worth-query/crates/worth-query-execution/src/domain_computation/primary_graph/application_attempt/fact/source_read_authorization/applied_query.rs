//! Successful admitted query READ support; no completion or positioned receipt is issued.
use super::*;
use crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof;
use worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity;

impl SourceReadAuthorization {
    pub(in crate::domain_computation::primary_graph) fn retain_applied_query(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        proof: &WorthQuerySessionGraphReadProof,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        source: [u8; 32],
        principal: EntityId,
        scope: EntityId,
        candidate: &RelationalAppliedCandidateReadIdentity,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        if !authorization.belongs_to_session(proof.session_identity()) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let support = SourceReadSupport::retain(execution, authorization, registry, allocation)?;
        SourceFactsAllocation::work(execution, 256).map_err(CompanionPreflightStop::Execution)?;
        let mut digest = Sha256::new();
        digest.update(b"worth-query:applied-query-read-authorization:v1");
        digest.update(source);
        for entity in [principal, scope] {
            digest.update(entity.partition_value().to_be_bytes());
            digest.update(entity.local_slot_value().to_be_bytes());
            digest.update(entity.generation_value().to_be_bytes());
        }
        let conventional = support.visit_support(&mut |bytes| {
            SourceFactsAllocation::work(execution, bytes.len() as u64)
                .map_err(CompanionPreflightStop::Execution)?;
            digest.update(bytes);
            Ok::<_, CompanionPreflightStop>(())
        })?;
        if !conventional {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let identity = digest.finalize().into();
        Ok(Arc::new(Self {
            key: SourceReadAuthorizationKey {
                source,
                support: identity,
                principal,
                scope,
            },
            binding: Some(binding),
            session: proof.session_identity(),
            support,
            identity,
            applied: Some(candidate.clone()),
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn principal_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.support.principal_facts()
    }

    pub(in crate::domain_computation::primary_graph) fn admits_applied_query(
        &self,
        proof: &WorthQuerySessionGraphReadProof,
        candidate: &RelationalAppliedCandidateReadIdentity,
    ) -> bool {
        self.binding.is_some()
            && self.session == proof.session_identity()
            && self.support.belongs_to_session(self.session)
            && self
                .applied
                .as_ref()
                .is_some_and(|actual| actual.qualification() == candidate.qualification())
    }
}
