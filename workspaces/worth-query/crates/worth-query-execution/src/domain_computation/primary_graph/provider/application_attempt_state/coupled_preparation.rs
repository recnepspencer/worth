//! One registered writer retains the source and handler it actually executed.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::AppliedSourcePublication, output_lineage::ProducerHandlerPreparation,
};
pub(in crate::domain_computation::primary_graph) enum CoupledAttemptPreparation {
    Producer(CoupledProducerPreparation),
    AuthoredStage {
        after_own: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        identity:
            worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        binding: &'static str,
    },
}
pub(in crate::domain_computation::primary_graph) struct CoupledProducerPreparation {
    pub(in crate::domain_computation::primary_graph::provider) completion:
        crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion,
    pub(in crate::domain_computation::primary_graph) after_own: Option<
        worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    pub(in crate::domain_computation::primary_graph::provider) handler: ProducerHandlerPreparation,
}
impl CoupledProducerPreparation {
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
    pub(in crate::domain_computation::primary_graph) fn retain_authored_stage_after_own(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        identity: worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        binding: &'static str,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        attempts
            .attempt_state_mut(session)
            .expect("actual registered authored stage")
            .attempt_mut()
            .retain_authored_stage_after_own(identity, binding, pin);
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
            });
        Ok(completion)
    }
}

/// The actual prepared Query session paired with its Native participant route.
/// This contains no executable input and cannot publish an independent proposal.
pub(in crate::domain_computation::primary_graph) struct CoupledApplicationMember {
    pub(in crate::domain_computation::primary_graph) position: usize,
    pub(in crate::domain_computation::primary_graph::provider) terminal:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub(in crate::domain_computation::primary_graph::provider) pending: Option<
        super::super::pending_application_publication::WorthQueryPendingApplicationPublication,
    >,
    pub(in crate::domain_computation::primary_graph::provider) owned:
        Option<OwnedCoupledContribution>,
    pub(in crate::domain_computation::primary_graph::provider) kind: CoupledApplicationMemberKind,
}
pub(in crate::domain_computation::primary_graph) enum CoupledApplicationMemberKind {
    Producer {
        producer: String,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    },
    AuthoredStage {
        stage: usize,
        identity:
            worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        source: worth_relational::facade::identity::EntityId,
        binding: &'static str,
    },
}
impl CoupledApplicationMember {
    pub(in crate::domain_computation::primary_graph) fn producer_route(
        &self,
    ) -> Option<(
        &str,
        &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    )> {
        match &self.kind {
            CoupledApplicationMemberKind::Producer { producer, meaning } => {
                Some((producer, meaning))
            }
            CoupledApplicationMemberKind::AuthoredStage { .. } => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        match &self.kind {
            CoupledApplicationMemberKind::Producer { meaning, .. } => meaning.source_scope,
            CoupledApplicationMemberKind::AuthoredStage { source, .. } => *source,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn from_authored_stage_session(
        terminal: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        stage: usize,
        identity: worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        source: worth_relational::facade::identity::EntityId,
        binding: &'static str,
    ) -> Self {
        Self {
            position: 0,
            terminal,
            owned: None,
            pending: None,
            kind: CoupledApplicationMemberKind::AuthoredStage {
                stage,
                identity,
                source,
                binding,
            },
        }
    }
    pub(in crate::domain_computation::primary_graph) fn from_prepared_session(
        terminal: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        producer: &str,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Self {
        Self {
            position: 0,
            terminal,
            owned: None,
            pending: None,
            kind: CoupledApplicationMemberKind::Producer {
                producer: producer.to_owned(),
                meaning,
            },
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
