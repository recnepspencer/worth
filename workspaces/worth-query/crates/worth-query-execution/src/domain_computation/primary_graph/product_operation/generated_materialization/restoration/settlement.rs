use worth_query_installation::facade::ApplicationSchema;

use super::recovery::{
    failure, RecoveryState, RestorationRetry, WorthQueryGeneratedOutputRestorationRecoveryFailure,
    WorthQueryGeneratedOutputRestorationRecoveryStage,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

pub(super) struct PublishedRestorationSettlement {
    pub(super) observation: worth_runtime_world::facade::ProductBranchObservation,
    pub(super) commit: worth_relational::facade::transactions::CommitResult,
    pub(super) retry: RestorationRetry,
}

impl PublishedRestorationSettlement {
    pub(super) fn finish<Schema: ApplicationSchema>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> super::WorthQueryRestoredGeneratedOutput {
        let Self {
            observation,
            commit,
            retry,
        } = self;
        let restored = retry
            .completion
            .complete(commit)
            .expect("World returns the exact prepared relational restoration result");
        runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .expect("application output lineage lock is available")
            .record_restoration(
                retry.producer.output_binding_type,
                retry.producer.runtime_authority,
                retry.producer.schema,
                retry.producer.scope,
                &observation,
                retry.correspondence,
                retry.producer.source_identity,
                retry.producer.observed_source_facts,
            );
        super::WorthQueryRestoredGeneratedOutput {
            branch: retry.branch,
            commit: super::WorthQueryGeneratedOutputRestorationReceipt::new(
                restored.commit.clone(),
            ),
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn finish_restoration_handoff(
        &self,
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        match self.release_product_publication_recovery(old, 0) {
            Ok(_) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
                RecoveryState::World {
                    recovery: next,
                    retry,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff,
                RecoveryState::HandoffWorld {
                    old: failed.into_recovery(),
                    next,
                    retry,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                RecoveryState::HandoffCleanup {
                    cleanup: failed.into_cleanup(),
                    next,
                    retry,
                },
            )),
        }
    }

    pub(super) fn finish_restoration_published(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedRestorationSettlement,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        match self.release_product_publication_recovery(recovery, 0) {
            Ok(_) => Ok(settlement.finish(self)),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff,
                RecoveryState::PublishedWorld {
                    recovery: failed.into_recovery(),
                    settlement,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                RecoveryState::PublishedCleanup {
                    cleanup: failed.into_cleanup(),
                    settlement,
                },
            )),
        }
    }
}
