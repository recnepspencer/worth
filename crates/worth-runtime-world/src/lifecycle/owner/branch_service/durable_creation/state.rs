//! Original durable branch creation, held only by the existing attempt row.
use crate::branch::registry::{ProductBranchInstallationWitness, ProductBranchRegistryReservation};
use crate::publication::durable_signal::{WorldDurableSignalAdvance, WorldDurableSignalBacking};
use crate::{branch::*, durability::*, identity::*, publication::*};
use std::sync::Arc;
use worth_relational::facade::branch::{
    AdmittedRelationalForkSourceBasis, RelationalCommittedTransitionEvidence,
    RelationalForkTargetReservation,
};
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum CreationPhase {
    Intent,
    Owners,
    Prepared,
    Releasing,
    Released,
    Selected,
    ReopenRequired,
    Performed,
    NoEffect,
}
#[derive(Debug)]
pub(crate) struct WorldDurableCreationCustody {
    pub(super) preparation_denial: Option<crate::lifecycle::WorldSignalForkPreparationDenial>,
    pub(super) completion: Option<Box<dyn crate::lifecycle::RuntimeWorldBranchCreationCompletion>>,
    pub(super) phase: CreationPhase,
    pub(super) binding: Arc<WorldDurableMediumBinding>,
    pub(super) subject: WorldDurableArtifact,
    pub(super) permit: Option<WorldProductCreationPermit>,
    pub(super) plan: LoweredBranchCreationPlan,
    pub(super) source: ProductBranchObservation,
    pub(super) branch: ProductBranchIdentity,
    pub(super) incarnation: ProductBranchIncarnation,
    pub(super) reservation: ProductBranchRegistryReservation,
    pub(super) witness: Arc<ProductBranchInstallationWitness>,
    pub(super) native_source: Option<AdmittedRelationalForkSourceBasis>,
    pub(super) native_target: Option<RelationalForkTargetReservation>,
    pub(super) native_original: Option<RelationalCommittedTransitionEvidence>,
    pub(super) native_caller: Option<worth_relational::facade::mvcc::RelationalCallerAssociation>,
    pub(super) native_custody: Option<ReservedCustodySlot>,
    pub(super) signal_custody: Option<ReservedCustodySlot>,
    pub(super) signal_prepared: Option<crate::lifecycle::PreparedWorldSignalFork>,
    pub(super) signal_backing: Option<WorldDurableSignalBacking>,
    pub(super) signal_task: Option<Arc<WorldDurableSignalAdvance>>,
    pub(crate) progress: CompositeAttemptProgress,
    pub(super) selected: Option<WorldDurableArtifact>,
    pub(super) release_subject: Option<WorldDurableArtifact>,
    pub(super) unsubmitted:
        Option<worth_proof::ProvenNoEffectPublication<super::release::UnsubmittedCreationOwners>>,
    pub(super) no_effect: Option<crate::lifecycle::RuntimeWorldBranchCreationNoEffect>,
    pub(super) released: Option<crate::lifecycle::RuntimeWorldBranchCreationRelease>,
    pub(super) origin: Option<PreparedCheckpointOccurrence>,
    pub(super) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorldDurableCreationCustody {
    pub(crate) fn preparation_denial(
        &self,
    ) -> Option<&crate::lifecycle::WorldSignalForkPreparationDenial> {
        self.preparation_denial.as_ref()
    }
    pub(crate) fn is_pending(&self) -> bool {
        !matches!(
            self.phase,
            CreationPhase::Performed | CreationPhase::NoEffect | CreationPhase::Released
        )
    }
    pub(crate) fn next_action(&self) -> crate::recovery::ProductUnpublishedNextAction {
        match self.phase {
            CreationPhase::Intent
            | CreationPhase::Owners
            | CreationPhase::Releasing
            | CreationPhase::Prepared
            | CreationPhase::Selected => {
                crate::recovery::ProductUnpublishedNextAction::ContinueOriginalPublication
            }
            CreationPhase::ReopenRequired => {
                crate::recovery::ProductUnpublishedNextAction::ReopenRequired
            }
            CreationPhase::Performed | CreationPhase::NoEffect | CreationPhase::Released => {
                unreachable!("terminal creation leaves its row")
            }
        }
    }
    pub(crate) fn destination_witness(&self) -> Arc<ProductBranchInstallationWitness> {
        self.witness.clone()
    }
}
