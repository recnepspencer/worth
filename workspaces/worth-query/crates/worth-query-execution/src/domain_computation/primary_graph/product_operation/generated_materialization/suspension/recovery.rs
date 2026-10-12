use crate::basis::WorthQueryProductBranchAdmissionDenial as Admission;
use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::RelationalMaterializationSuspensionCompletion;

use super::super::{ProducerQualification, WorthQuerySuspendedGeneratedOutput};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

#[path = "recovery/settlement.rs"]
mod settlement;
use settlement::PublishedSuspensionSettlement;

#[path = "recovery/stage.rs"]
mod stage;
pub use stage::WorthQueryGeneratedOutputSuspensionRecoveryStage;

/// Custody of a suspension whose publication moved some owners but not the
/// product head.
///
/// Continue it with `continue_generated_output_suspension_recovery` until it
/// returns the suspended output. It holds the exact custody needed to finish.
#[must_use = "suspension recovery retains exact World and Relational custody"]
pub struct WorthQueryGeneratedOutputSuspensionRecovery {
    state: RecoveryState,
}

enum RecoveryState {
    Unpublished {
        product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
        retry: SuspensionRetry,
    },
    World {
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
    },
    HandoffWorld {
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
    },
    HandoffCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        next: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
    },
    PublishedWorld {
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedSuspensionSettlement,
    },
    PublishedCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        settlement: PublishedSuspensionSettlement,
    },
}

impl RecoveryState {
    fn retry(&self) -> &SuspensionRetry {
        match self {
            Self::Unpublished { retry, .. }
            | Self::World { retry, .. }
            | Self::HandoffWorld { retry, .. }
            | Self::HandoffCleanup { retry, .. } => retry,
            Self::PublishedWorld { settlement, .. } | Self::PublishedCleanup { settlement, .. } => {
                &settlement.retry
            }
        }
    }
}

struct SuspensionRetry {
    completion: RelationalMaterializationSuspensionCompletion,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: ProducerQualification,
}

impl WorthQueryGeneratedOutputSuspensionRecovery {
    pub(in crate::domain_computation::primary_graph::product_operation::generated_materialization) fn new(
        product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
        completion: RelationalMaterializationSuspensionCompletion,
        branch: crate::basis::WorthQueryProductBranch,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: ProducerQualification,
    ) -> Self {
        Self {
            state: RecoveryState::Unpublished {
                product,
                retry: SuspensionRetry {
                    completion,
                    branch,
                    correspondence,
                    producer,
                },
            },
        }
    }

    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        match &self.state {
            RecoveryState::Unpublished { retry, .. }
            | RecoveryState::World { retry, .. }
            | RecoveryState::HandoffWorld { retry, .. }
            | RecoveryState::HandoffCleanup { retry, .. } => retry.branch,
            RecoveryState::PublishedWorld { settlement, .. }
            | RecoveryState::PublishedCleanup { settlement, .. } => settlement.retry.branch,
        }
    }
}

/// A recovery step that did not finish: the stage it stopped at, and the
/// recovery handed back to continue.
pub struct WorthQueryGeneratedOutputSuspensionRecoveryFailure {
    stage: WorthQueryGeneratedOutputSuspensionRecoveryStage,
    recovery: WorthQueryGeneratedOutputSuspensionRecovery,
}

impl WorthQueryGeneratedOutputSuspensionRecoveryFailure {
    pub const fn stage(&self) -> WorthQueryGeneratedOutputSuspensionRecoveryStage {
        self.stage
    }

    pub fn into_recovery(self) -> WorthQueryGeneratedOutputSuspensionRecovery {
        self.recovery
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn continue_generated_output_suspension_recovery<Producer>(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let mut retained = Some(recovery);
        let result = self.with_application_advancement(request, |phase| {
            let phase = &phase;
            let recovery = retained.take().expect("one admitted recovery");

            if !retry_matches::<Schema, Producer>(self, recovery.state.retry()) {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
                    recovery.state,
                ));
            }
            match recovery.state {
                RecoveryState::Unpublished { product, retry } => {
                    let needs_settlement = product.relational_requires_settlement();
                    self.continue_world(
                        phase,
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
                                WorthQueryGeneratedOutputSuspensionRecoveryStage::Inspection,
                                RecoveryState::World { recovery, retry },
                            ));
                        }
                    };
                    self.continue_world(phase, recovery, retry, needs_settlement, request)
                }
                RecoveryState::HandoffWorld { old, next, retry } => {
                    self.finish_handoff(old, next, retry)
                }
                RecoveryState::HandoffCleanup {
                    cleanup,
                    next,
                    retry,
                } => match cleanup.retry() {
                    Ok(_) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
                        RecoveryState::World {
                            recovery: next,
                            retry,
                        },
                    )),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
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
                } => self.finish_published(recovery, settlement),
                RecoveryState::PublishedCleanup {
                    cleanup,
                    settlement,
                } => match cleanup.retry() {
                    Ok(_) => Ok(settlement.finish()),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
                        RecoveryState::PublishedCleanup {
                            cleanup: failed.into_cleanup(),
                            settlement,
                        },
                    )),
                },
            }
        });
        match result {
            Ok(result) => result,
            Err(cause) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(cause),
                retained.expect("refusal precedes admission").state,
            )),
        }
    }

    fn continue_world(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
        needs_settlement: bool,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        if needs_settlement && recovery.continue_owner_settlement().is_err() {
            return Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerSettlement,
                RecoveryState::World { recovery, retry },
            ));
        }
        self.adopt_settled(phase, recovery, retry, request)
    }

    fn adopt_settled(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        let source_read = {
            let _read = self.product_runtime.enter_named_read(crate::domain_computation::execution_runtime::product_world::read_consult::NamedRead::GeneratedRecoverySourceSelection);
            self.on_branch(retry.branch).select(phase)
        };
        let selected = match source_read {
            Ok(selected) => selected,
            Err(cause) => {
                return Err(failure(
                    match cause {
                        cause @ Admission::Stopped(_) => {
                            WorthQueryGeneratedOutputSuspensionRecoveryStage::SourceSelection(cause)
                        }
                        Admission::Refused(_) => {
                            WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence
                        }
                    },
                    RecoveryState::World { recovery, retry },
                ));
            }
        };
        let observation = selected.product().observation();
        if observation.lifecycle_incarnation() != retry.producer.output_occurrence
            || observation.reference_generation().get() != retry.producer.output_generation
        {
            return Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
                RecoveryState::World { recovery, retry },
            ));
        }
        let publication_execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(cause.into()),
                    RecoveryState::World { recovery, retry },
                ))
            }
        };
        let unpublished = match recovery.inspect() {
            Ok(unpublished) => unpublished,
            Err(_) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::Inspection,
                    RecoveryState::World { recovery, retry },
                ));
            }
        };
        let binding = selected.product().publication_binding();
        let prepared = match binding.prepare_settled_relational_adoption(&unpublished, request) {
            Ok(prepared) => prepared,
            Err(PublicationDenial::Stopped(stop)) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(
                        advancement_denial(stop),
                    ),
                    RecoveryState::World { recovery, retry },
                ))
            }
            Err(PublicationDenial::Refused(_)) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPreparation,
                    RecoveryState::World { recovery, retry },
                ))
            }
        };
        drop(unpublished);
        match prepared.execute(publication_execution) {
            Ok(PublicationOutcome::Performed(performed)) => {
                let commit = performed
                    .component_results()
                    .relational_commit_result()
                    .cloned()
                    .expect("settled adoption carries the exact Relational result");
                let mut publication = performed.consume();
                let observation = publication
                    .take_successor_observation()
                    .expect("settled adoption reserves its successor observation");
                self.finish_published(
                    recovery,
                    PublishedSuspensionSettlement {
                        publication: self.materialization_publication_binding(observation),
                        commit,
                        retry,
                    },
                )
            }
            Err(PublicationDenial::Stopped(stop)) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(
                    advancement_denial(stop),
                ),
                RecoveryState::World { recovery, retry },
            )),
            Err(PublicationDenial::Refused(_)) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
                RecoveryState::World { recovery, retry },
            )),
            Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                let next = self
                    .unpublished_materialization_from_binding(effects, &binding)
                    .into_recovery();
                self.finish_handoff(recovery, next, retry)
            }
        }
    }
}

fn failure(
    stage: WorthQueryGeneratedOutputSuspensionRecoveryStage,
    state: RecoveryState,
) -> WorthQueryGeneratedOutputSuspensionRecoveryFailure {
    WorthQueryGeneratedOutputSuspensionRecoveryFailure {
        stage,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery { state },
    }
}

mod retry_identity;
use retry_identity::retry_matches;
