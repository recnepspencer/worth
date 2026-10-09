//! Exact continuation of World unpublished transport completion effects.

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::{
    InstalledTransportCompletion, InstalledTransportPendingReason,
    PerformedInstalledTransportCompletion, RecoveryProgress, RecoveryRelease,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecovery,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn finish_recovery_release(
        &self,
        release: RecoveryRelease,
    ) -> Result<(), RecoveryRelease> {
        match release {
            RecoveryRelease::World(recovery) => self
                .release_product_publication_recovery(recovery, 0)
                .map(|_| ())
                .map_err(|failure| match failure {
                    WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed) => {
                        RecoveryRelease::World(failed.into_recovery())
                    }
                    WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed) => {
                        RecoveryRelease::Cleanup(failed.into_cleanup())
                    }
                }),
            RecoveryRelease::Cleanup(cleanup) => cleanup
                .retry()
                .map(|_| ())
                .map_err(|failed| RecoveryRelease::Cleanup(failed.into_cleanup())),
        }
    }

    pub(super) fn continue_transport_recovery(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        evidence: Arc<InstalledTransportCompletion>,
        prior: WorthQueryProductUnpublishedRecovery,
        request: &WorthQueryRequestScope,
    ) -> RecoveryProgress {
        use InstalledTransportPendingReason as Pending;
        let owner = evidence.committed();
        let incarnation = owner.committed_product_publication().product_incarnation();
        let lane = self
            .primary_provider
            .application_branch_commit_lane_for_occurrence(incarnation);
        let lane = match lane {
            Ok(lane) => lane,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::PublicationAtCapacity)
            }
        };
        let _coordination = lane.enter();
        let binding = match self.resolve_installed_transport_completion_binding(owner) {
            Ok(binding) => binding,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
        };
        let _publication_permit = match self.reserve_installed_transport_publication(&binding) {
            Ok(permit) => permit,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::PublicationAtCapacity)
            }
        };
        let needs_settlement = match prior.inspect() {
            Ok(unpublished) => unpublished.relational_requires_settlement(),
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
        };
        if needs_settlement && prior.continue_owner_settlement().is_err() {
            return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired);
        }
        let unpublished = match prior.inspect() {
            Ok(unpublished) => unpublished,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
        };
        let lease = match self.product_runtime.admit_product_occurrence(incarnation) {
            Ok(lease) => lease,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
        };
        let publication = lease.publication_binding();
        if unpublished.expected_product() != publication.observation() {
            drop(unpublished);
            drop(publication);
            drop(lease);
            return match self.finish_recovery_release(RecoveryRelease::World(prior)) {
                Ok(()) => RecoveryProgress::StaleReleased,
                Err(failed) => RecoveryProgress::StalePending(failed),
            };
        }
        let prepared = match publication.prepare_settled_relational_adoption(&unpublished, request)
        {
            Ok(prepared) => prepared,
            Err(_) => {
                return RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
        };
        let receipt = WorthQueryReservedProductPublicationReceipt::new(
            publication.root_identity(),
            prepared.unpublished_recovery_handle(),
            false,
            false,
            false,
        );
        let world_recovery = publication.recovery();
        drop(unpublished);
        let outcome = prepared.execute(
            phase
                .execution_request_for(&self.product_runtime)
                .expect("private progression uses its admitted runtime phase"),
        );
        drop(publication);
        drop(lease);
        match outcome {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                let terminal = PerformedInstalledTransportCompletion::new(
                    evidence,
                    binding,
                    incarnation,
                    receipt.fill(performed.consume(), None),
                );
                let release = self
                    .finish_recovery_release(RecoveryRelease::World(prior))
                    .err();
                RecoveryProgress::Performed(terminal, release)
            }
            RuntimeWorldPublicationOutcome::NoEffect(_) => {
                RecoveryProgress::Pending(prior, None, Pending::ProductRecoveryRequired)
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                let next = WorthQueryProductUnpublishedApplication::new(
                    effects,
                    world_recovery,
                    self.primary_provider.unpublished_idempotency_disposition(),
                )
                .into_recovery();
                let release = self
                    .finish_recovery_release(RecoveryRelease::World(prior))
                    .err();
                RecoveryProgress::Pending(next, release, Pending::ProductRecoveryRequired)
            }
        }
    }
}
