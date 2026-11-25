//! Admit and execute one restoration inside its original application phase.
use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn restore_generated_output_in_phase<Producer>(
        &self,
        completed: WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let execution = match phase.execution_request_for(self.inner.runtime.product_runtime_ref())
        {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(restoration_failure(
                    completed.suspended,
                    WorthQueryGeneratedOutputRestorationFailureCause::advancement(cause.into()),
                ))
            }
        };

        let WorthQueryCompletedGeneratedOutputReconstruction {
            suspended,
            entities,
            relations,
            marker: _,
        } = completed;
        if !std::sync::Arc::ptr_eq(
            &suspended.publication.root_identity(),
            &self.inner.runtime.product_runtime_ref().root_identity(),
        ) || !suspended.matches_runtime(self)
        {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::ForeignRuntime,
            ));
        }
        if !suspended.matches_producer_binding::<Schema, Producer>() {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::WrongProducer,
            ));
        }
        if !suspended.matches_provider_version::<Schema, Producer>()
            || self
                .inner
                .installed_producers
                .provider::<Producer>()
                .is_none()
        {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::StaleProducerVersion,
            ));
        }
        if !suspended.matches_retained_lineage::<Schema, Producer>(self) {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::StaleOutputLineage,
            ));
        }
        let gate = match self
            .inner
            .runtime
            .product_runtime_ref()
            .activations_ref()
            .gate(suspended.publication.observation().branch_identity())
        {
            Ok(gate) => gate,
            Err(_) => {
                return Err(restoration_failure(
                    suspended,
                    WorthQueryGeneratedOutputRestorationFailureCause::ProductActivationUnavailable,
                ));
            }
        };
        let _publication_admission = match gate.begin_publication() {
            Ok(admission) => admission,
            Err(_) => {
                return Err(restoration_failure(
                    suspended,
                    WorthQueryGeneratedOutputRestorationFailureCause::ProductActivationUnavailable,
                ));
            }
        };
        let WorthQuerySuspendedGeneratedOutput {
            publication,
            branch,
            custody,
            correspondence,
            producer,
        } = suspended;
        let pending = self
            .inner
            .runtime
            .primary_provider_ref()
            .graph_ref()
            .with_runtime(&execution, |runtime| {
                runtime
                    .owner_component_services()
                    .materialization_port()
                    .validate_generated_rematerialization(
                        &execution,
                        publication.observation().basis().relational_basis(),
                        custody,
                        entities,
                        relations,
                    )
            });
        let pending = match pending {
            Ok(pending) => pending,
            Err(failure) => {
                let cause = preparation_failure_cause(&failure.error);
                return Err(restoration_failure(
                    WorthQuerySuspendedGeneratedOutput {
                        publication,
                        branch,
                        custody: failure.custody,
                        correspondence,
                        producer,
                    },
                    cause,
                ));
            }
        };
        let (proposal, pending_completion, invariant_evidence) = pending.into_parts();
        if let Err(denial) = invariant_admission::admit::<Schema, Producer>(
            &invariant_evidence,
            publication
                .observation()
                .basis()
                .relational_basis()
                .identity()
                .branch_id(),
        ) {
            drop(proposal);
            return Err(restoration_failure(
                WorthQuerySuspendedGeneratedOutput {
                    publication,
                    branch,
                    custody: pending_completion.into_custody(),
                    correspondence,
                    producer,
                },
                WorthQueryGeneratedOutputRestorationFailureCause::InvariantAdmission(denial),
            ));
        }
        self.publish_restoration(
            &execution,
            phase,
            publication,
            branch,
            correspondence,
            producer,
            proposal,
            pending_completion,
            request,
        )
    }
}
