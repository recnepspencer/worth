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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputRestorationRecoveryStage {
    Inspection,
    OwnerSettlement,
    AdoptionPreparation,
    AdoptionPublication,
    RecoveryHandoff,
    OwnerCleanup,
    StaleOccurrence,
}

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
    pub(super) completion: RelationalRematerializationCompletion,
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
}

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
    pub fn continue_generated_output_restoration_recovery(
        &self,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        if !restoration_retry_matches(self, recovery.state.retry()) {
            return Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::StaleOccurrence,
                recovery.state,
            ));
        }
        match recovery.state {
            RecoveryState::Unpublished { unpublished } => {
                let (product, retry) = unpublished.into_parts();
                let needs_settlement = product.relational_requires_settlement();
                self.continue_restoration_world(
                    product.into_recovery(),
                    retry,
                    needs_settlement,
                    request,
                )
            }
            RecoveryState::World { recovery, retry } => {
                let needs_settlement = match recovery.inspect() {
                    Ok(product) => product.relational_requires_settlement(),
                    Err(_) => {
                        return Err(failure(
                            WorthQueryGeneratedOutputRestorationRecoveryStage::Inspection,
                            RecoveryState::World { recovery, retry },
                        ));
                    }
                };
                self.continue_restoration_world(recovery, retry, needs_settlement, request)
            }
            RecoveryState::HandoffWorld { old, next, retry } => {
                self.finish_restoration_handoff(old, next, retry)
            }
            RecoveryState::HandoffCleanup {
                cleanup,
                next,
                retry,
            } => match cleanup.retry() {
                Ok(_) => Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
                    RecoveryState::World {
                        recovery: next,
                        retry,
                    },
                )),
                Err(failed) => Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                    RecoveryState::HandoffCleanup {
                        cleanup: failed.into_cleanup(),
                        next,
                        retry,
                    },
                )),
            },
            RecoveryState::PublishedWorld {
                recovery,
                settlement,
            } => self.finish_restoration_published(recovery, settlement),
            RecoveryState::PublishedCleanup {
                cleanup,
                settlement,
            } => match cleanup.retry() {
                Ok(_) => Ok(settlement.finish(self)),
                Err(failed) => Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                    RecoveryState::PublishedCleanup {
                        cleanup: failed.into_cleanup(),
                        settlement,
                    },
                )),
            },
        }
    }

    fn continue_restoration_world(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
        needs_settlement: bool,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
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
        self.adopt_settled_restoration(recovery, retry, request)
    }

    fn adopt_settled_restoration(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
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
        let prepared = match retry
            .publication
            .prepare_settled_relational_adoption(&unpublished, request)
        {
            Ok(prepared) => prepared,
            Err(_) => {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPreparation,
                    RecoveryState::World { recovery, retry },
                ));
            }
        };
        drop(unpublished);
        let publication = retry.publication.clone();
        match prepared.execute() {
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) => {
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
                    recovery,
                    PublishedRestorationSettlement {
                        observation,
                        commit,
                        retry,
                    },
                )
            }
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::NoEffect(_) => {
                Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
                    RecoveryState::World { recovery, retry },
                ))
            }
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(
                effects,
            ) => {
                let next = self
                    .unpublished_materialization_from_binding(effects, &publication)
                    .into_recovery();
                self.finish_restoration_handoff(recovery, next, retry)
            }
        }
    }
}

fn restoration_retry_matches<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retry: &RestorationRetry,
) -> bool {
    retry.producer.runtime_authority == runtime.runtime.authority_identity().as_u64()
        && retry.producer.schema == runtime.installed_schema.binding_identity()
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
