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
        let shared = match self.prepare_decision_query_basis(phase, admission.graph_work()) {
            Ok(shared) => shared,
            Err(cause) => {
                return WorthQueryMutationHandlerExecutionReport::new(
                    Err(cause),
                    WorthQueryMutationHandlerWork::NotStarted,
                )
            }
        };
        let request = admission.publication_request().clone();
        let access = SourceQueryReaderAccess {
            application: self,
            shared: &shared,
            principal,
            request: &request,
        };
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
        phase: &WorthQueryAdvancementPhase<'_>,
        work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<
        crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation<
            '_,
            S,
        >,
        MutationHandlerExecutionDenial,
    > {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(MutationHandlerExecutionDenial::ForeignAdvancementPhase)?;
        work.mutation_lease()
            .ok_or(DecisionQueryPreparationStop::MissingProduct)
            .and_then(|lease| {
                self.on_mutation_product(&execution, lease)
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
