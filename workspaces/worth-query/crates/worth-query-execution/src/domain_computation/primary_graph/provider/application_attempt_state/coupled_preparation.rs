//! One registered writer retains the source and handler it actually executed.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::AppliedSourcePublication, output_lineage::ProducerHandlerPreparation,
};
pub(in crate::domain_computation::primary_graph) struct CoupledProducerPreparation {
    pub(in crate::domain_computation::primary_graph::provider) completion:
        crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion,
    pub(in crate::domain_computation::primary_graph::provider) after_own: Option<
        worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    pub(in crate::domain_computation::primary_graph::provider) handler: ProducerHandlerPreparation,
    pub(in crate::domain_computation::primary_graph::provider) query_support:
        Vec<crate::domain_computation::primary_graph::application_query::AppliedQuerySupport>,
}
impl CoupledProducerPreparation {
    pub(in crate::domain_computation::primary_graph) fn retain_query_support(
        &mut self,
        support: crate::domain_computation::primary_graph::application_query::AppliedQuerySupport,
    ) {
        self.query_support.push(support);
    }
    pub(in crate::domain_computation::primary_graph) fn retain_after_own(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        assert!(self.after_own.replace(pin).is_none());
    }
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_query_support(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        support: crate::domain_computation::primary_graph::application_query::AppliedQuerySupport,
    ) {
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let state = attempts
            .attempt_state_mut(session)
            .expect("actual registered discovery parent");
        state.attempt_mut().retain_coupled_query_support(support);
    }
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_after_own(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let state = attempts
            .attempt_state_mut(session)
            .expect("actual writer before Native append");
        state.attempt_mut().retain_coupled_after_own(pin);
    }
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_preparation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        source: AppliedSourcePublication,
        handler: ProducerHandlerPreparation,
    ) -> Result<
        crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion,
        crate::domain_computation::WorthQueryInvariantExecutionFailure,
    > {
        let completion = crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion::prepare(execution, source).map_err(|cause| crate::domain_computation::WorthQueryInvariantExecutionFailure::new(crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected, cause.subject()))?;
        let mut attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
        let state = attempts.attempt_state_mut(session).ok_or_else(||
            crate::domain_computation::WorthQueryInvariantExecutionFailure::new(
                crate::domain_computation::WorthQueryInvariantExecutionDenialKind::StateLoadClosureMismatch,
                "coupled preparation lost its exact registered writer"))?;
        state
            .attempt_mut()
            .retain_coupled_preparation(CoupledProducerPreparation {
                completion: completion.clone(),
                handler,
                after_own: None,
                query_support: Vec::new(),
            });
        Ok(completion)
    }
}

/// The actual prepared Query session paired with its Native participant route.
/// This contains no executable input and cannot publish an independent proposal.
pub(in crate::domain_computation::primary_graph) struct CoupledApplicationMember {
    pub(in crate::domain_computation::primary_graph::provider) terminal:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub(in crate::domain_computation::primary_graph::provider) pending: Option<
        super::super::pending_application_publication::WorthQueryPendingApplicationPublication,
    >,
    pub(in crate::domain_computation::primary_graph::provider) owned:
        Option<OwnedCoupledContribution>,
    pub(in crate::domain_computation::primary_graph::provider) producer: String,
    pub(in crate::domain_computation::primary_graph::provider) meaning:
        worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
}
impl CoupledApplicationMember {
    pub(in crate::domain_computation::primary_graph) fn from_prepared_session(
        terminal: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        producer: &str,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Self {
        Self {
            terminal,
            owned: None,
            pending: None,
            producer: producer.to_owned(),
            meaning,
        }
    }
}

/// The registered member moves out with the root under the same attempt gate.
pub(in crate::domain_computation::primary_graph::provider) struct OwnedCoupledContribution {
    pub(in crate::domain_computation::primary_graph::provider) attempt:
        super::super::WorthQueryPrimaryGraphApplicationAttempt,
    pub(in crate::domain_computation::primary_graph::provider) work:
        super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
    pub(in crate::domain_computation::primary_graph::provider) commit:
        Option<super::commit_preparation::CoupledCommitPreparation>,
    pub(super) _completion: super::commit_completion::WorthQueryApplicationAttemptCompletion,
}
