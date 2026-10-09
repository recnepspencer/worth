//! Re-admits the retained adoption facts inside the caller's active advancement.
use super::*;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(super) fn recover_branch_adoption_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryBranchAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
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
            Err(denial) => {
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
        match prepared.execute(
            phase
                .execution_request_for(&self.application().product_runtime)
                .expect("private recovery uses its admitted runtime phase"),
        ) {
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) => {
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
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
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
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(
                effects,
            ) => {
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
