//! Authorization from the actual applied Source READ, never a positioned terminal.
use super::*;
use sha2::{Digest, Sha256};
pub(in crate::domain_computation::primary_graph) trait SourceReadAuthorizationAppliedExt: Sized {
    fn validates_applied_terminal(
        &self,
        terminal: &crate::domain_computation::provider_session::WorthQueryAppliedSourceRead,
    ) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn complete_applied(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        terminal: &crate::domain_computation::provider_session::WorthQueryAppliedSourceRead,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        source: [u8; 32],
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Option<Arc<Self>>, CompanionPreflightStop>;
}

impl SourceReadAuthorizationAppliedExt for SourceReadAuthorization {
    fn validates_applied_terminal(
        &self,
        terminal: &crate::domain_computation::provider_session::WorthQueryAppliedSourceRead,
    ) -> bool {
        self.binding.is_some()
            && self.session == terminal.session_identity()
            && self.support.belongs_to_session(self.session)
            && self.key.principal == terminal.principal()
            && self.key.scope == terminal.scope()
            && self.applied.as_ref().is_some_and(|candidate| {
                candidate.qualification() == terminal.candidate().qualification()
            })
    }

    /// Only the genuine applied READ terminal retains this independent authority.
    fn complete_applied(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        terminal: &crate::domain_computation::provider_session::WorthQueryAppliedSourceRead,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        source: [u8; 32],
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Option<Arc<Self>>, CompanionPreflightStop> {
        if !authorization.belongs_to_session(terminal.session_identity()) {
            return Ok(None);
        }
        let support = SourceReadSupport::retain(execution, authorization, registry, allocation)?;
        SourceFactsAllocation::work(execution, 256).map_err(CompanionPreflightStop::Execution)?;
        let mut digest = Sha256::new();
        digest.update(b"worth-query:source-read-authorization:v1");
        digest.update(source);
        for entity in [terminal.principal(), terminal.scope()] {
            digest.update(entity.partition_value().to_be_bytes());
            digest.update(entity.local_slot_value().to_be_bytes());
            digest.update(entity.generation_value().to_be_bytes());
        }
        if !support.visit_support(&mut |bytes| {
            SourceFactsAllocation::work(execution, bytes.len() as u64)
                .map_err(CompanionPreflightStop::Execution)?;
            digest.update(bytes);
            Ok::<_, CompanionPreflightStop>(())
        })? {
            return Ok(None);
        }
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let identity = digest.finalize().into();
        Ok(Some(Arc::new(Self {
            key: SourceReadAuthorizationKey {
                source,
                support: identity,
                principal: terminal.principal(),
                scope: terminal.scope(),
            },
            binding: Some(binding),
            session: terminal.session_identity(),
            support: Box::new(support),
            identity,
            applied: Some(terminal.candidate().clone()),
        })))
    }
}
