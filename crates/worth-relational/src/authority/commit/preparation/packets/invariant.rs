use std::sync::Arc;

use crate::authority::commit::preparation::planning::context::PreparationPlanningContext;
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::PreparationLocalityProof;
use crate::authority::commit::preparation::proofs::validity::PreparationProofValidity;
use crate::authority::commit::preparation::reduction::keys::ValidationReductionKey;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::{
    CustomInvariantRegistration, InvariantRegistration, PreparedCustomInvariantExecution,
};
use crate::validation::engine::{InvariantObservation, PreparedRelationIntegrityScopes};

#[derive(Clone)]
pub(crate) enum InvariantPacketRegistration {
    Native(InvariantRegistration),
    Custom {
        registration: CustomInvariantRegistration,
        prepared_execution: Arc<dyn PreparedCustomInvariantExecution>,
        prepared_scope: crate::validation::data::PreparedCustomInvariantScope,
    },
    CustomNotApplicable {
        registration: CustomInvariantRegistration,
        prepared_scope: crate::validation::data::PreparedCustomInvariantScope,
        work: crate::validation::CustomInvariantWorkMeter,
    },
}

impl InvariantPacketRegistration {
    pub(crate) fn execution_point(&self) -> crate::validation::data::InvariantExecutionPoint {
        match self {
            Self::Native(registration) => registration.execution_point,
            Self::Custom { registration, .. } | Self::CustomNotApplicable { registration, .. } => {
                registration.execution_point()
            }
        }
    }

    pub(crate) fn failure_effect(&self) -> crate::validation::data::InvariantFailureEffect {
        match self {
            Self::Native(registration) => registration.failure_effect,
            Self::Custom { registration, .. } | Self::CustomNotApplicable { registration, .. } => {
                registration.failure_effect()
            }
        }
    }

    pub(crate) fn groups(&self) -> crate::validation::data::InvariantGroupSet {
        match self {
            Self::Native(registration) => registration.groups(),
            Self::Custom { registration, .. } | Self::CustomNotApplicable { registration, .. } => {
                registration.groups()
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct InvariantWorkPacket<'runtime> {
    pub(crate) packet_index: usize,
    pub(crate) registration: InvariantPacketRegistration,
    pub(crate) reduction_key: ValidationReductionKey,
    pub(crate) proof_kind: PreparationProofKind,
    pub(crate) locality: PreparationLocalityProof,
    pub(crate) validity: PreparationProofValidity,
    pub(crate) planning_context: Arc<PreparationPlanningContext>,
    pub(crate) observation: &'runtime InvariantObservation<'runtime>,
    pub(crate) version_id: crate::identity::data::VersionId,
    pub(crate) current_version_id: crate::identity::data::VersionId,
    pub(crate) merged_plan: Option<&'runtime MergedCommitPlan>,
    pub(crate) relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    pub(crate) current_version_minimum_index:
        Arc<std::sync::OnceLock<crate::validation::engine::evaluator::CurrentVersionMinimumIndex>>,
}
