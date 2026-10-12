//! Re-admits the retained adoption facts inside the caller's active advancement.
use super::*;
use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn recover_branch_adoption_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        let execution = match recovery_execution(phase, &self.application().product_runtime) {
            Ok(execution) => execution,
            Err(denial) => return Err(failure(denial, recovery)),
        };
        let inspected = match recovery.product.inspect() {
            Ok(inspected) => inspected,
            Err(denial) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(denial),
                    recovery,
                ))
            }
        };
        if inspected.expected_product() != self.product().publication_binding().observation() {
            drop(inspected);
            return Err(failure(
                WorthQueryBranchAdoptionRecoveryDenial::ProductAffinityMismatch,
                recovery,
            ));
        }
        let needs_settlement = inspected.relational_requires_settlement();
        drop(inspected);
        if needs_settlement {
            if let Err(denial) = recovery.product.continue_owner_settlement() {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::OwnerSettlement(denial),
                    recovery,
                ));
            }
        }
        let unpublished = match recovery.product.inspect() {
            Ok(unpublished) => unpublished,
            Err(denial) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(denial),
                    recovery,
                ))
            }
        };
        let prepared = match self
            .product()
            .publication_binding()
            .prepare_settled_relational_adoption(&unpublished, request)
        {
            Ok(prepared) => prepared,
            Err(PublicationDenial::Stopped(stop)) => {
                drop(unpublished);
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(advancement_denial(
                        stop,
                    )),
                    recovery,
                ));
            }
            Err(PublicationDenial::Refused(denial)) => {
                drop(unpublished);
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::AdoptionPreparation(denial),
                    recovery,
                ));
            }
        };
        drop(unpublished);
        let binding = self.product().publication_binding().clone();
        let WorthQueryBranchAdoptionRecovery {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        } = recovery;
        match prepared.execute(execution) {
            Ok(PublicationOutcome::Performed(performed)) => {
                let adoption = WorthQueryPerformedBranchAdoption::new(
                    performed.consume(),
                    source,
                    target,
                    selected_entity_count,
                    migration,
                    custody,
                );
                let cleanup = self.application().release_product_publication_recovery(
                    product,
                    IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS,
                );
                Ok(WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup })
            }
            Err(PublicationDenial::Stopped(stop)) => Err(failure(
                WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied(advancement_denial(stop)),
                WorthQueryBranchAdoptionRecovery {
                    source,
                    target,
                    selected_entity_count,
                    migration,
                    custody,
                    product,
                    support_custody,
                },
            )),
            Err(PublicationDenial::Refused(no_effect)) => {
                Ok(WorthQueryBranchAdoptionRecoveryOutcome::NoEffect {
                    no_effect,
                    recovery: WorthQueryBranchAdoptionRecovery {
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                        product,
                        support_custody,
                    },
                })
            }
            Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                let next = WorthQueryUnpublishedBranchAdoption::new(
                    source,
                    target,
                    selected_entity_count,
                    migration,
                    custody,
                    effects,
                    binding.recovery(),
                    self.application()
                        .primary_provider
                        .unpublished_idempotency_disposition(),
                    support_custody,
                );
                let prior_cleanup = self.application().release_product_publication_recovery(
                    product,
                    IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS,
                );
                Ok(
                    WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished {
                        next,
                        prior_cleanup,
                    },
                )
            }
        }
    }
}

fn recovery_execution<'scope>(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
    runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
) -> Result<worth_execution::ExecutionRequest<'scope, 'scope>, WorthQueryBranchAdoptionRecoveryDenial>
{
    phase
        .checked_execution_request_for(runtime)
        .map_err(WorthQueryBranchAdoptionRecoveryDenial::ExecutionDenied)
}

#[cfg(test)]
#[path = "foreign_phase_tests.rs"]
mod foreign_phase_tests;

impl crate::domain_computation::execution_observation::WorthQueryExecutionObservation {
    /// Drives the private continuation with an already selected product and
    /// real retained recovery custody; the listener owns this test door.
    #[doc(hidden)]
    pub fn recover_branch_adoption_in_advancement_for_test<Schema: ApplicationSchema>(
        &self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        selected.recover_branch_adoption_in_advancement(phase, recovery, request)
    }
}
