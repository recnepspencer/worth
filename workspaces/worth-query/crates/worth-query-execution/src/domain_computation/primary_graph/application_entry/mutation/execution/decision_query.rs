//! Ordinary authenticated decisions borrow Query reads at their exact admitted product.
use super::*;
use crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;

#[derive(Debug)]
enum DecisionQueryPreparationStop {
    MissingProduct,
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Native(worth_relational::facade::mvcc::CompanionPreflightStop),
}
impl std::fmt::Display for DecisionQueryPreparationStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DecisionQueryPreparationStop {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Product(cause) => Some(cause),
            Self::Native(_) | Self::MissingProduct => None,
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn native_decision_cause<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a worth_relational::facade::mvcc::CompanionPreflightStop> {
    match error.downcast_ref::<DecisionQueryPreparationStop>()? {
        DecisionQueryPreparationStop::Native(cause) => Some(cause),
        _ => None,
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Runs an authenticated handler with its declared Parent reads on the
    /// product selected by this admission and the admission's original request.
    /// Every Parent and Source still perform their own current READ checks.
    pub fn execute_mutation_handler_report_with_query_reads<B: ApplicationMutationBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        principal_identity: &B::PrincipalIdentity,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        admission: WorthQueryAdmittedApplicationOperation<
            S,
            B::Operation,
            B::Input,
            <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
        >,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryMutationHandlerExecutionReport<
        WorthQueryCompletedMutationCandidate<S, B>,
        B::Denial,
    > {
        let refused = |cause| {
            WorthQueryMutationHandlerExecutionReport::new(
                Err(cause),
                WorthQueryMutationHandlerWork::NotStarted,
            )
        };
        // Ownership is refused before the branch, so neither path does work
        // for a foreign core; the phase check keeps preparation's order.
        let execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return refused(MutationHandlerExecutionDenial::ForeignAdvancementPhase(
                    cause,
                ))
            }
        };
        if let Err(denial) = admission.validate_runtime_ownership(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return refused(MutationHandlerExecutionDenial::Projection(denial.into()));
        }
        if admission
            .allowed_graph_contract()
            .managed_index_reads()
            .is_empty()
        {
            return self.execute_mutation_handler_report::<B>(
                phase,
                identities,
                principal_identity,
                admission,
                allocation,
            );
        }
        // A forged tag type is refused before the shared basis is prepared.
        if let Err(denial) = admission.validate_installed_mutation_binding(
            self.installed_schema
                .installed_mutation_binding_descriptor(B::IDENTITY),
        ) {
            return refused(denial);
        }
        let shared = match self.prepare_decision_query_basis(&execution, admission.graph_work()) {
            Ok(shared) => shared,
            Err(cause) => return refused(cause),
        };
        let request = admission.publication_request().clone();
        let access = SourceQueryReaderAccess::ordinary(self, &shared, principal, &request);
        match self.prepare_mutation_handler_plan_with_source_access::<B>(
            phase,
            identities,
            principal_identity,
            Some(&access),
            admission,
            || {},
            allocation,
        ) {
            Ok(prepared) => prepared.run(self, identities),
            Err(report) => report,
        }
    }
    fn prepare_decision_query_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<
        crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation<
            '_,
            S,
        >,
        MutationHandlerExecutionDenial,
    > {
        work.mutation_lease()
            .ok_or(DecisionQueryPreparationStop::MissingProduct)
            .and_then(|lease| {
                self.on_mutation_product(execution, lease)
                    .map_err(DecisionQueryPreparationStop::Product)
            })
            .and_then(|selected| {
                selected
                    .prepare_shared_query_basis()
                    .map_err(|(_, cause)| DecisionQueryPreparationStop::Native(cause))
            })
            .map_err(|cause| {
                MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause))
            })
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;
    #[test]
    fn handler_denial_preserves_native_decision_preparation_cause() {
        let denial = HandlerExecutionDenial::new(DecisionQueryPreparationStop::Native(
            worth_relational::facade::mvcc::CompanionPreflightStop::ManagedSourceChanged,
        ));
        assert!(matches!(
            denial.native_decision_refusal(),
            Some(worth_relational::facade::mvcc::CompanionPreflightStop::ManagedSourceChanged)
        ));
        let retained = denial
            .source
            .as_ref()
            .downcast_ref::<DecisionQueryPreparationStop>()
            .unwrap();
        let DecisionQueryPreparationStop::Native(original) = retained else {
            unreachable!()
        };
        assert!(std::ptr::eq(
            denial.native_decision_refusal().unwrap(),
            original
        ));
    }
}
