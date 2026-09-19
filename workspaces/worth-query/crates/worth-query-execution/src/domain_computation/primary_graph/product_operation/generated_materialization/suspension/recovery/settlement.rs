use worth_query_installation::facade::ApplicationSchema;

use super::{
    failure, RecoveryState, SuspensionRetry, WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    WorthQueryGeneratedOutputSuspensionRecoveryStage,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

pub(super) struct PublishedSuspensionSettlement {
    pub(super) publication:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    pub(super) commit: worth_relational::facade::transactions::CommitResult,
    pub(super) retry: SuspensionRetry,
}

impl PublishedSuspensionSettlement {
    pub(super) fn finish(self) -> super::WorthQuerySuspendedGeneratedOutput {
        let Self {
            publication,
            commit,
            retry,
        } = self;
        super::super::super::suspended_output(
            publication,
            retry.branch,
            retry.correspondence,
            retry.producer,
            retry.completion,
            commit,
        )
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn finish_handoff(
        &self,
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
    ) -> Result<
        super::WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        match self.release_product_publication_recovery(old, 0) {
            Ok(_) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
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
                WorthQueryGeneratedOutputSuspensionRecoveryStage::RecoveryHandoff,
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
                WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
                RecoveryState::HandoffCleanup {
                    cleanup: failed.into_cleanup(),
                    next,
                    retry,
                },
            )),
        }
    }

    pub(super) fn finish_published(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedSuspensionSettlement,
    ) -> Result<
        super::WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        match self.release_product_publication_recovery(recovery, 0) {
            Ok(_) => Ok(settlement.finish()),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::RecoveryHandoff,
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
                WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
                RecoveryState::PublishedCleanup {
                    cleanup: failed.into_cleanup(),
                    settlement,
                },
            )),
        }
    }
}
