use crate::domain_computation::execution_runtime::product_world::{
    OwnerCleanupRetirements, ProductPublicationBindingPrimaryExt,
};
mod advancement;
use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::RelationalRematerializationCompletion;

use super::super::ProducerQualification;
use super::settlement::PublishedRestorationSettlement;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductPublicationBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

/// The step at which continuing a restoration recovery stopped. The recovery is
/// handed back to continue again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputRestorationRecoveryStage {
    /// The request stopped while continuing retained recovery.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The unpublished product state could not be inspected.
    Inspection,
    /// Owner settlement of the unpublished effects did not finish.
    OwnerSettlement,
    /// Adopting the settled effects could not be prepared.
    AdoptionPreparation,
    /// World retained an early durable open before Native preparation completed
    /// and requires Home reopen before the original occurrence can be resolved.
    ReopenRequired,
    /// World retained an early attempt before Native preparation completed and
    /// allows releasing the World recovery record directly via
    /// `into_world_recovery()`.
    ReleaseRequired,
    /// The adoption is not published yet; continue again.
    AdoptionPublication,
    /// Releasing an earlier recovery authority did not finish.
    RecoveryHandoff,
    /// Owner cleanup did not finish.
    OwnerCleanup,
    /// The recovery belongs to another runtime or schema binding.
    StaleOccurrence,
}

/// Custody of a restoration whose publication moved some owners but not the
/// product head.
///
/// Continue it with `continue_generated_output_restoration_recovery` until it
/// returns the restored output. It holds the exact custody needed to finish.
#[must_use = "restoration recovery retains exact World and Relational custody"]
pub struct WorthQueryGeneratedOutputRestorationRecovery {
    state: RecoveryState,
}

pub(super) enum RecoveryState {
    Unpublished {
        unpublished: super::WorthQueryUnpublishedGeneratedOutputRestoration,
    },
    World {
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
    },
    HandoffWorld {
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
    },
    HandoffCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        next: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
    },
    PublishedWorld {
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedRestorationSettlement,
    },
    PublishedCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        settlement: PublishedRestorationSettlement,
    },
}

impl RecoveryState {
    fn retry(&self) -> &RestorationRetry {
        match self {
            Self::Unpublished { unpublished } => &unpublished.retry,
            Self::World { retry, .. }
            | Self::HandoffWorld { retry, .. }
            | Self::HandoffCleanup { retry, .. } => retry,
            Self::PublishedWorld { settlement, .. } | Self::PublishedCleanup { settlement, .. } => {
                &settlement.retry
            }
        }
    }
}

pub(super) struct RestorationRetry {
    pub(super) publication: WorthQueryProductPublicationBinding,
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) producer: ProducerQualification,
    pub(super) completion: Option<RelationalRematerializationCompletion>,
}

impl WorthQueryGeneratedOutputRestorationRecovery {
    pub(super) fn from_unpublished(
        unpublished: super::WorthQueryUnpublishedGeneratedOutputRestoration,
    ) -> Self {
        Self {
            state: RecoveryState::Unpublished { unpublished },
        }
    }

    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        match &self.state {
            RecoveryState::Unpublished { unpublished } => unpublished.retry.branch,
            RecoveryState::World { retry, .. }
            | RecoveryState::HandoffWorld { retry, .. }
            | RecoveryState::HandoffCleanup { retry, .. } => retry.branch,
            RecoveryState::PublishedWorld { settlement, .. }
            | RecoveryState::PublishedCleanup { settlement, .. } => settlement.retry.branch,
        }
    }

    pub fn next_actions(
        &self,
    ) -> Result<
        Vec<worth_runtime_world::facade::ProductUnpublishedNextAction>,
        worth_runtime_world::facade::RuntimeWorldRecoveryDenial,
    > {
        match &self.state {
            RecoveryState::Unpublished { unpublished } => Ok(unpublished.next_actions().to_vec()),
            RecoveryState::World { recovery, .. }
            | RecoveryState::HandoffWorld { next: recovery, .. }
            | RecoveryState::HandoffCleanup { next: recovery, .. }
            | RecoveryState::PublishedWorld { recovery, .. } => {
                Ok(recovery.inspect()?.next_actions().to_vec())
            }
            RecoveryState::PublishedCleanup { .. } => Ok(Vec::new()),
        }
    }

    /// Hand back World's unpublished recovery handle when this recovery holds a
    /// single unpublished World record (`Unpublished` or `World`). States with
    /// handoff or pending cleanup custody return `Err(self)` intact.
    pub fn into_world_recovery(
        self,
    ) -> Result<WorthQueryProductUnpublishedRecovery, Self> {
        match self.state {
            RecoveryState::Unpublished { unpublished } => Ok(unpublished.into_world_recovery()),
            RecoveryState::World { recovery, .. } => Ok(recovery),
            state @ (RecoveryState::HandoffWorld { .. }
            | RecoveryState::HandoffCleanup { .. }
            | RecoveryState::PublishedWorld { .. }
            | RecoveryState::PublishedCleanup { .. }) => Err(Self { state }),
        }
    }
}

/// A recovery step that did not finish: the stage it stopped at, and the
/// recovery handed back to continue.
pub struct WorthQueryGeneratedOutputRestorationRecoveryFailure {
    stage: WorthQueryGeneratedOutputRestorationRecoveryStage,
    recovery: WorthQueryGeneratedOutputRestorationRecovery,
}

impl WorthQueryGeneratedOutputRestorationRecoveryFailure {
    pub const fn stage(&self) -> WorthQueryGeneratedOutputRestorationRecoveryStage {
        self.stage
    }

    pub fn into_recovery(self) -> WorthQueryGeneratedOutputRestorationRecovery {
        self.recovery
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn continue_restoration_world(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
        needs_settlement: bool,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        if needs_settlement && recovery.continue_owner_settlement().is_err() {
            return Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerSettlement,
                RecoveryState::World { recovery, retry },
            ));
        }
        self.adopt_settled_restoration(execution, recovery, retry, request, retirements)
    }

    fn adopt_settled_restoration(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        let unpublished = match recovery.inspect() {
            Ok(unpublished) => unpublished,
            Err(_) => {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::Inspection,
                    RecoveryState::World { recovery, retry },
                ));
            }
        };
        if unpublished.next_actions().contains(
            &worth_runtime_world::facade::ProductUnpublishedNextAction::ReopenRequired,
        ) {
            return Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::ReopenRequired,
                RecoveryState::World { recovery, retry },
            ));
        }
        if retry.completion.is_none() {
            let stage = early_restoration_stage(&unpublished);
            return Err(failure(stage, RecoveryState::World { recovery, retry }));
        }
        let prepared = match retry
            .publication
            .prepare_settled_relational_adoption(&unpublished, request)
        {
            Ok(prepared) => prepared,
            Err(PublicationDenial::ClockMismatch) => {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(
                        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock),
                    ),
                    RecoveryState::World { recovery, retry },
                ))
            },
Err(PublicationDenial::Stopped(stop)) => {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(
                        advancement_denial(stop),
                    ),
                    RecoveryState::World { recovery, retry },
                ))
            }
            Err(PublicationDenial::Refused(_)) => {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPreparation,
                    RecoveryState::World { recovery, retry },
                ))
            }
        };
        drop(unpublished);
        let publication = retry.publication.clone();
        match prepared.execute(*execution) {
            Ok(PublicationOutcome::Performed(performed)) => {
                let commit = performed
                    .component_results()
                    .relational_commit_result()
                    .cloned()
                    .expect("settled adoption carries the exact Relational result");
                let mut performed = performed.consume();
                let observation = performed
                    .take_successor_observation()
                    .expect("settled restoration adoption reserves its successor observation");
                self.finish_restoration_published(
                    execution,
                    recovery,
                    PublishedRestorationSettlement {
                        observation,
                        commit,
                        retry,
                    },
                    retirements,
                )
            }
            Err(PublicationDenial::ClockMismatch) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock),
                ),
                RecoveryState::World { recovery, retry },
            )),
Err(PublicationDenial::Stopped(stop)) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(
                    advancement_denial(stop),
                ),
                RecoveryState::World { recovery, retry },
            )),
            Err(PublicationDenial::Refused(_)) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
                RecoveryState::World { recovery, retry },
            )),
            Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                let next = self
                    .unpublished_materialization_from_binding(effects, &publication)
                    .into_recovery();
                self.finish_restoration_handoff(execution, recovery, next, retry, retirements)
            }
        }
    }
}

fn restoration_retry_matches<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retry: &RestorationRetry,
) -> bool {
    retry.producer.runtime_authority == runtime.inner.runtime.authority_identity().as_u64()
        && retry.producer.schema == runtime.inner.installed_schema.binding_identity()
}

fn early_restoration_stage(
    unpublished: &crate::domain_computation::WorthQueryProductUnpublishedApplication,
) -> WorthQueryGeneratedOutputRestorationRecoveryStage {
    use worth_runtime_world::facade::ProductUnpublishedNextAction as Action;
    let actions = unpublished.next_actions();
    if actions.contains(&Action::ReopenRequired) {
        WorthQueryGeneratedOutputRestorationRecoveryStage::ReopenRequired
    } else if actions.contains(&Action::ReleaseObligations) {
        WorthQueryGeneratedOutputRestorationRecoveryStage::ReleaseRequired
    } else {
        WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPreparation
    }
}

pub(super) fn failure(
    stage: WorthQueryGeneratedOutputRestorationRecoveryStage,
    state: RecoveryState,
) -> WorthQueryGeneratedOutputRestorationRecoveryFailure {
    WorthQueryGeneratedOutputRestorationRecoveryFailure {
        stage,
        recovery: WorthQueryGeneratedOutputRestorationRecovery { state },
    }
}
