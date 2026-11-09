//! Bind exact applied source facts without creating a committed source epoch.
use super::super::AppliedQuerySource;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

impl<Query, Value> AppliedQuerySource<Query, Value> {
    pub(in crate::domain_computation::primary_graph) fn bind_applied_expectation<
        Schema,
        Binding,
        Scope,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        input: &Binding::Input,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<WorthQueryBoundSourceExpectation, WorthQuerySourceExpectationDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let denied = || {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                Binding::IDENTITY,
            )
        };
        let checked =
            runtime.checked_source_contract::<Binding, Scope>(admission, input, |expected| {
                self.admitted_parameters
                    .matches_expected(expected)
                    .map_err(|cause| parameter_denial(Binding::IDENTITY, cause))
            })?;
        let lease = admission.graph_work().mutation_lease().ok_or_else(denied)?;
        let expected_query = runtime
            .installed_schema
            .installed_query_identity_by_name(checked.expected_query_identifier)
            .ok_or_else(denied)?;
        if self.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || self.query != *expected_query.as_bytes()
            || self.root() != admission.scope_entity_id()
            || self.candidate.qualification() != pin.candidate_identity().qualification()
            || self.candidate.proposal().branch_observation()
                != lease.product().relational_basis().reference()
        {
            return Err(denied());
        }
        let facts = super::super::fact_conversion::applied_facts(
            self.meaning.footprint(),
            checked.layout,
            Binding::IDENTITY,
            SourceFactExpansionControl::in_request(
                *execution,
                Some(admission.publication_request()),
            ),
        )?;
        let bound = WorthQueryBoundSourceExpectation {
            kind: WorthQuerySourceExpectationKind::Row,
            identity: self.idempotency_identity(),
            partition_identity: self.partition(),
        };
        admission.bind_source_partition(bound.partition_identity);
        admission.bind_source_expectation_kind(bound.kind);
        admission.bind_source_facts(facts, None);
        admission
            .graph_work_mut()
            .mutation_lease_mut()
            .expect("checked genuine before lease")
            .bind_applied_source_projection(pin.clone());
        Ok(bound)
    }
}
