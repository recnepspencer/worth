//! Checked source ownership retained across recorded idempotency resolution.
use super::*;

/// A checked source identity with its actual observed-source owner.
/// No fact payload, allocation policy, or lease is stored in this phase.
pub struct WorthQueryPendingSourceExpectation<Query> {
    pub(super) admission_identity: WorthQueryOperationAdmissionIdentity,
    pub(super) selected_product: crate::basis::WorthQueryProductBranchReadIdentity,
    pub(super) expected_query_identifier: &'static str,
    pub(super) bound: WorthQueryBoundSourceExpectation,
    pub(super) source: WorthQueryObservedSource<Query>,
}

impl<Query> WorthQueryPendingSourceExpectation<Query> {
    /// The same checked extension used for fresh execution and recorded retries.
    pub const fn bound_source(&self) -> WorthQueryBoundSourceExpectation {
        self.bound
    }

    /// Materialize only after recorded resolution has selected fresh execution.
    pub fn consume_into<Schema, Operation, Input, Scope>(
        self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQuerySourceExpectationDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        if self.admission_identity != admission.admission_identity()
            || !admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| {
                    self.selected_product
                        .matches_observation(product.observation())
                })
        {
            return Err(WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignOperation,
                "pending source expectation",
            ));
        }
        let graph = runtime.runtime.primary_graph().ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::ForeignApplication,
                self.expected_query_identifier,
            )
        })?;
        let query_identity = runtime
            .installed_schema
            .installed_query_identity_by_name(self.expected_query_identifier)
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    self.expected_query_identifier,
                )
            })?;
        self.source.validate_affinity(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            admission.graph_work_branch(),
            admission.scope_entity_id(),
            &self.selected_product,
            self.expected_query_identifier,
            query_identity,
        )?;
        let control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(admission.publication_request(), allocation_policy);
        let facts = self.source.validated_facts(
            &graph.layout,
            self.expected_query_identifier,
            Some(admission.publication_request()),
            control.policy(),
        )?;
        admission.bind_source_facts(facts);
        Ok(())
    }
}
