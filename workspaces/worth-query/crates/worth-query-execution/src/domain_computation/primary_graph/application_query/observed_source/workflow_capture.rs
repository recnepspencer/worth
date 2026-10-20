//! Native source correspondence without premature guarded-operation admission.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding,
        ApplicationMutationSourceExpectation,
    },
    application_query::ApplicationQueryParameterSet,
};
type Scope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type Query<S, B> = <<B as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query;

impl<S: worth_query_installation::facade::ApplicationSchema>
    WorthQuerySelectedProductOperation<'_, S>
{
    pub(in crate::domain_computation::primary_graph) fn describe_workflow_source<B>(
        &self,
        scope: &WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
        input: &B::Input,
        source: &WorthQueryObservedSource<Query<S, B>>,
    ) -> Result<([u8; 32], [u8; 32]), WorthQuerySourceExpectationDenial>
    where
        B: ApplicationMutationBinding<S>,
    {
        let deny = |kind| WorthQuerySourceExpectationDenial::new(kind, B::IDENTITY);
        let query = B::SourceExpectation::QUERY_IDENTIFIER
            .ok_or_else(|| deny(WorthQuerySourceExpectationDenialKind::SourceContractMismatch))?;
        let runtime = self.application();
        let identity = runtime
            .installed_schema
            .installed_query_identity_by_name(query)
            .ok_or_else(|| deny(WorthQuerySourceExpectationDenialKind::SourceContractMismatch))?;
        let installed = runtime
            .installed_schema
            .installed_mutation_binding::<B>()
            .map_err(|_| deny(WorthQuerySourceExpectationDenialKind::SourceContractMismatch))?;
        let body_context = installed
            .operation()
            .contracts()
            .body()
            .is_some_and(|body| body.requires_query_context());
        if body_context
            && !source
                .parameters
                .matches_expected(ApplicationQueryParameterSet::<Query<S, B>>::new())
                .map_err(|cause| prepared_expectation::parameter_denial(B::IDENTITY, cause))?
        {
            return Err(deny(
                WorthQuerySourceExpectationDenialKind::SourceParametersMismatch,
            ));
        }
        if let Some(expected) = B::expected_source_parameters(input)
            .map_err(|_| deny(WorthQuerySourceExpectationDenialKind::SourceParametersMismatch))?
            && !source
                .parameters
                .matches_expected(expected)
                .map_err(|cause| prepared_expectation::parameter_denial(B::IDENTITY, cause))?
        {
            return Err(deny(
                WorthQuerySourceExpectationDenialKind::SourceParametersMismatch,
            ));
        }
        source.validate_affinity(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            self.application_basis().identity().branch_id(),
            scope.entity_id(),
            &crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            ),
            query,
            identity,
        )?;
        Ok((
            source.idempotency_identity().bytes(),
            source.partition_identity(),
        ))
    }
}
