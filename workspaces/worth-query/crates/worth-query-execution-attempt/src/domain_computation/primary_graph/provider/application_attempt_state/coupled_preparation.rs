//! One registered writer retains the source and handler it actually executed.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::AppliedSourcePublication, output_lineage::ProducerHandlerPreparation,
};
pub enum CoupledAttemptPreparation {
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
pub struct CoupledProducerPreparation {
    pub completion:
        crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion,
    pub outputs: Option<AppliedParticipantOutputs>,
    pub after_own: Option<
        worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    pub handler: ProducerHandlerPreparation,
}
pub struct AppliedParticipantOutputs {
    pub producer: String,
    pub meaning:
        worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    pub entities:
        worth_execution::ExecutionArray<worth_relational::facade::identity::EntityId>,
}
impl CoupledProducerPreparation {
    pub fn retain_after_own(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        outputs: AppliedParticipantOutputs,
    ) {
        assert!(self.outputs.replace(outputs).is_none());
        assert!(self.after_own.replace(pin).is_none());
    }
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub fn retain_authored_stage_after_own(
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
    pub fn retain_coupled_after_own(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        outputs: AppliedParticipantOutputs,
    ) {
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let state = attempts
            .attempt_state_mut(session)
            .expect("actual writer before Native append");
        state.attempt_mut().retain_coupled_after_own(pin, outputs);
    }
    pub fn retain_coupled_preparation(
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
                outputs: None,
            });
        Ok(completion)
    }
}

/// The actual prepared Query session paired with its Native participant route.
/// This contains no executable input and cannot publish an independent proposal.
pub struct CoupledApplicationMember {
    pub position: usize,
    /// For a producer: the applied position of the stage whose effect owed
    /// this run. Work owed before a reader's position settles before it.
    pub owed: Option<usize>,
    /// For a producer: the verified parents its derivation consumed.
    pub consumes: Vec<super::ParentRead>,
    /// For a producer: its own derived output, as its consumers carry it.
    pub produces: Option<super::ParentRead>,
    pub terminal:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    pub pending: Option<
        super::super::pending_application_publication::WorthQueryPendingApplicationPublication,
    >,
    pub owned:
        Option<OwnedCoupledContribution>,
    pub kind: CoupledApplicationMemberKind,
}
pub enum CoupledApplicationMemberKind {
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
    pub fn extracted_attempt(
        &self,
    ) -> &super::super::WorthQueryPrimaryGraphApplicationAttempt {
        &self
            .owned
            .as_ref()
            .expect("extracted registered member")
            .attempt
    }
    pub fn producer_route(
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
    /// Sequentially this member runs after the stage at `reader`: an
    /// authored stage applied later, or a producer owed by that stage or a
    /// later one. A producer owed earlier belongs before the reader.
    pub fn settles_after(
        &self,
        reader: usize,
    ) -> bool {
        match &self.kind {
            CoupledApplicationMemberKind::AuthoredStage { .. } => self.position > reader,
            CoupledApplicationMemberKind::Producer { .. } => {
                self.owed.is_some_and(|owed| owed >= reader)
            }
        }
    }
    pub fn source(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        match &self.kind {
            CoupledApplicationMemberKind::Producer { meaning, .. } => meaning.source_scope,
            CoupledApplicationMemberKind::AuthoredStage { source, .. } => *source,
        }
    }
    pub fn from_authored_stage_session(
        terminal: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        stage: usize,
        identity: worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity,
        source: worth_relational::facade::identity::EntityId,
        binding: &'static str,
    ) -> Self {
        Self {
            position: 0,
            owed: None,
            consumes: Vec::new(),
            produces: None,
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
    pub fn from_prepared_session(
        terminal: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
        producer: &str,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    ) -> Self {
        Self {
            position: 0,
            owed: None,
            consumes: Vec::new(),
            produces: None,
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
pub struct OwnedCoupledContribution {
    pub attempt:
        super::super::WorthQueryPrimaryGraphApplicationAttempt,
    pub work:
        super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
    pub commit:
        Option<super::commit_preparation::CoupledCommitPreparation>,
    pub(super) _completion: super::commit_completion::WorthQueryApplicationAttemptCompletion,
}
