//! Original World publication and candidate-affine recovery retain exact D.
use super::*;
use crate::domain_computation::primary_graph::product_operation::ProductOperationContextScopedOps;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationDenial as PublicationDenial,
    crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as Outcome,
    crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication,
    crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch,
    crate::domain_computation::request_execution::advancement_denial,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
    crate::domain_computation::WorthQueryProductUnpublishedApplication,
    crate::domain_computation::WorthQueryProductUnpublishedRecovery,
};

pub(super) fn publication_denial(
    cause: PublicationDenial<worth_runtime_world::facade::NoEffectCompositePublication>,
) -> WorthQueryHistoryNavigationDenial {
    match cause {
        PublicationDenial::ClockMismatch => {
            WorthQueryHistoryNavigationDenial::Stopped(crate::domain_computation::request_execution::WorthQueryAdvancementDenial::Resource(crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock))
        },
PublicationDenial::Stopped(cause) => {
            WorthQueryHistoryNavigationDenial::Stopped(advancement_denial(cause))
        }
        PublicationDenial::Refused(cause) => WorthQueryHistoryNavigationDenial::Publication(cause),
    }
}

impl<'runtime, S: ApplicationSchema> NavigationProgress<'runtime, S> {
    pub(super) fn publish(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        publication: WorthQueryPreparedProductPublication,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        let execution =
            match phase.checked_execution_request_for(self.application.product_runtime()) {
                Ok(execution) => execution,
                Err(cause) => {
                    self.phase = NavigationPhase::Publication {
                        candidate,
                        publication,
                    };
                    return Err(WorthQueryHistoryNavigationDenial::Stopped(cause));
                }
            };
        match publication.execute(execution) {
            Ok(Outcome::Performed(performed)) => {
                self.performed(candidate, performed);
                Ok(())
            }
            Ok(Outcome::ProductUnpublished(effects)) => {
                let recovery = self.unpublished(effects);
                self.phase = NavigationPhase::Unpublished {
                    candidate,
                    recovery,
                };
                Ok(())
            }
            Err(cause) => {
                self.phase = NavigationPhase::Candidate(candidate);
                Err(publication_denial(cause))
            }
        }
    }

    fn performed(
        &mut self,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        performed: worth_runtime_world::facade::PerformedCompositePublication,
    ) {
        let mut publication = performed.consume();
        let observation = publication.take_successor_observation().expect("initial and settled-adoption preparations both reserve the exact successor observation");
        debug_assert_eq!(
            observation.lifecycle_incarnation(),
            candidate.branch().occurrence()
        );
        debug_assert_eq!(
            observation.selected_commit(),
            publication.commit().identity()
        );
        let exact = crate::basis::WorthQueryProductObservationLease::new(observation);
        if self.output_profile.is_some() {
            self.output_source = Some(crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation::from_product_with_memory(
                self.application.application_runtime(), exact.retained_clone(), self.output_source_memory.take(),
            ));
        }
        self.phase = if self.output_profile.is_some() {
            NavigationPhase::AwaitingOutputs {
                candidate,
                exact,
                _publication: publication,
            }
        } else {
            NavigationPhase::Selecting {
                candidate,
                exact,
                _publication: publication,
                _outputs: None,
            }
        };
    }

    fn unpublished(
        &self,
        effects: worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
    ) -> WorthQueryProductUnpublishedRecovery {
        WorthQueryProductUnpublishedApplication::new(
            effects,
            self.application.product_runtime().owner.recovery_port(),
            self.application
                .primary_provider()
                .unpublished_idempotency_disposition(),
        )
        .into_recovery()
    }

    pub(super) fn recover(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        recovery: WorthQueryProductUnpublishedRecovery,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let prepared = (|| {
            let execution = phase
                .checked_execution_request_for(self.application.product_runtime())
                .map_err(Denial::Stopped)?;
            let product = self
                .application
                .product_runtime()
                .admit_product_occurrence(candidate.branch().occurrence())
                .map_err(Denial::ProductAdmission)?;
            let selected = self
                .application
                .on_product_for_owner_recovery(&execution, product)
                .map_err(Denial::ProductAdmission)?;
            let inspected = recovery.inspect().map_err(Denial::Recovery)?;
            if inspected.expected_product()
                != selected.product().publication_binding().observation()
            {
                return Err(Denial::RecoveryAffinity);
            }
            let settlement = inspected.relational_requires_settlement();
            drop(inspected);
            if settlement {
                recovery
                    .continue_owner_settlement()
                    .map_err(Denial::Recovery)?;
            }
            let inspected = recovery.inspect().map_err(Denial::Recovery)?;
            selected
                .product()
                .publication_binding()
                .prepare_settled_relational_adoption(&inspected, request)
                .map_err(|cause| match cause {
                    PublicationDenial::ClockMismatch => Denial::Stopped(crate::domain_computation::request_execution::WorthQueryAdvancementDenial::Resource(crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock)),
PublicationDenial::Stopped(cause) => Denial::Stopped(advancement_denial(cause)),
                    PublicationDenial::Refused(cause) => Denial::RecoveryPreparation(cause),
                })
        })();
        let publication = match prepared {
            Ok(publication) => publication,
            Err(cause) => {
                self.phase = NavigationPhase::Unpublished {
                    candidate,
                    recovery,
                };
                return Err(cause);
            }
        };
        let execution =
            match phase.checked_execution_request_for(self.application.product_runtime()) {
                Ok(execution) => execution,
                Err(cause) => {
                    drop(publication);
                    self.phase = NavigationPhase::Unpublished {
                        candidate,
                        recovery,
                    };
                    return Err(Denial::Stopped(cause));
                }
            };
        match publication.execute(execution) {
            Ok(Outcome::Performed(performed)) => {
                self.performed(candidate, performed);
                self.prior_cleanup = self
                    .application
                    .application_runtime()
                    .release_product_publication_recovery_in_advancement(phase, recovery, 0)
                    .err();
                Ok(())
            }
            Ok(Outcome::ProductUnpublished(effects)) => {
                let next = self.unpublished(effects);
                self.phase = NavigationPhase::Unpublished {
                    candidate,
                    recovery: next,
                };
                self.prior_cleanup = self
                    .application
                    .application_runtime()
                    .release_product_publication_recovery_in_advancement(phase, recovery, 0)
                    .err();
                Ok(())
            }
            Err(cause) => {
                self.phase = NavigationPhase::Unpublished {
                    candidate,
                    recovery,
                };
                Err(publication_denial(cause))
            }
        }
    }
}
