//! Ordinary authenticated decisions borrow Query reads at their exact admitted product.

use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use super::*;
use crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_entry::mutation::execution::decision_query::DecisionQueryPreparationStop;

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
        let execution = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return refused(MutationHandlerExecutionDenial::ForeignAdvancementPhase(
                    cause,
                ))
            }
        };
        if let Err(denial) = admission.validate_mutation_entry(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
            &self.runtime.installed_operations,
        ) {
            return refused(denial);
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
        if let Err(denial) = admission.validate_installed_mutation_binding(
            self.installed_schema
                .installed_mutation_binding_descriptor(B::IDENTITY),
        ) {
            return refused(denial);
        }
        if let Err(denial) = admission.validate_current_authority() {
            return refused(MutationHandlerExecutionDenial::Projection(denial.into()));
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
            Err(report) => report.into_uncompleted_report(),
        }
    }

    /// An authored stage retains C authorization while its ordinary readers
    /// borrow the exact unpublished D pin from Native.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_authored_stage_handler<B: ApplicationMutationBinding<S>>(
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
        before: &crate::basis::WorthQueryProductBranchLease,
        applied: Option<(
            &worth_relational::facade::runtime::RelationalPinnedRead,
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
            &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
        )>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
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
        let execution = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return refused(MutationHandlerExecutionDenial::ForeignAdvancementPhase(
                    cause,
                ))
            }
        };
        let shared = match self.prepare_decision_query_basis(&execution, admission.graph_work()) {
            Ok(shared) => shared,
            Err(cause) => return refused(cause),
        };
        if shared.selected().product().observation() != before.observation() {
            return refused(MutationHandlerExecutionDenial::Handler(
                HandlerExecutionDenial::new(DecisionQueryPreparationStop::MissingProduct),
            ));
        }
        let request = admission.publication_request().clone();
        let operation_principal = admission.operation_scope_binding().principal();
        let admitted_request = admission.retain_admitted_request();
        let access = SourceQueryReaderAccess::ordinary(self, &shared, principal, &request)
            .with_coordinated_control(
                crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadControl::new(
                    shared.selected().product().read_lease_ref(),
                    &admitted_request,
                    &operation_principal,
                ),
                coordination,
            );
        let access = match access {
            Ok(access) => access,
            Err(cause) => {
                return refused(MutationHandlerExecutionDenial::Handler(
                    HandlerExecutionDenial::new(cause),
                ))
            }
        };
        let (applied, access) = match applied {
            Some((before_pin, candidate, participants)) => (
                Some((before_pin, candidate)),
                access.with_applied_participants(participants),
            ),
            None => (None, access),
        };
        let report = match self.prepare_mutation_handler_plan_on_basis::<B>(
            phase,
            identities,
            principal_identity,
            Some(&access),
            admission,
            applied,
            None,
            || {},
            allocation,
        ) {
            Ok(prepared) => prepared.run(self, identities),
            Err(report) => report.into_uncompleted_report(),
        };
        if let Some(cause) = access.take_control_failure() {
            return refused(MutationHandlerExecutionDenial::Handler(
                HandlerExecutionDenial::new(cause),
            ));
        }
        report
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
        use crate::domain_computation::provider_session::ManagedGraphWorkSessionPrimaryExt;
        work.mutation_lease_typed()
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
