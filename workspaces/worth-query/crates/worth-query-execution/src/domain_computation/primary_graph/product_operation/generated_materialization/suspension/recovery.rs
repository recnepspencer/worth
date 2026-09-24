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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputSuspensionRecoveryStage {
    Inspection,
    OwnerSettlement,
    AdoptionPreparation,
    AdoptionPublication,
    RecoveryHandoff,
    OwnerCleanup,
    StaleOccurrence,
}

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
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        if !retry_matches::<Schema, Producer>(self, recovery.state.retry()) {
            return Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
                recovery.state,
            ));
        }
        match recovery.state {
            RecoveryState::Unpublished { product, retry } => {
                let needs_settlement = product.relational_requires_settlement();
                self.continue_world(product.into_recovery(), retry, needs_settlement, request)
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
                self.continue_world(recovery, retry, needs_settlement, request)
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
    }

    fn continue_world(
        &self,
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
        self.adopt_settled(recovery, retry, request)
    }

    fn adopt_settled(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        let selected = match self.on_branch(retry.branch).select() {
            Ok(selected) => selected,
            Err(_) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
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
            Err(_) => {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPreparation,
                    RecoveryState::World { recovery, retry },
                ));
            }
        };
        drop(unpublished);
        match prepared.execute() {
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) => {
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
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::NoEffect(_) => {
                Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
                    RecoveryState::World { recovery, retry },
                ))
            }
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(
                effects,
            ) => {
                let next = self
                    .unpublished_materialization_from_binding(effects, &binding)
                    .into_recovery();
                self.finish_handoff(recovery, next, retry)
            }
        }
    }
}

fn retry_matches<Schema, Producer>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retry: &SuspensionRetry,
) -> bool
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    if retry.producer.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || retry.producer.schema != runtime.installed_schema.binding_identity()
        || retry.producer.binding_type != std::any::TypeId::of::<Producer>()
        || retry.producer.binding_identity != Producer::IDENTITY
        || retry.producer.provider_identity != Producer::Provider::SEMANTIC_IDENTITY
        || runtime.installed_producers.provider::<Producer>().is_none()
    {
        return false;
    }
    runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .expect("application output lineage lock is available")
        .qualified_output::<Producer::Operation>(
            retry.producer.runtime_authority,
            &retry.producer.schema,
            retry.producer.scope,
            retry.producer.output_occurrence,
            retry.producer.output_generation,
            retry.producer.runtime_source_identity,
            retry.producer.checkpoint_source_identity,
        )
        .is_some_and(|exact| {
            exact.source_identity == retry.producer.recorded_source_identity
                && exact.runtime_authority == retry.producer.runtime_authority
                && exact.schema == retry.producer.schema
                && exact.scope == retry.producer.scope
                && Arc::ptr_eq(&exact.correspondence, &retry.correspondence)
        })
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
