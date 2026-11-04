//! Pending materialization requires the same validation-owned unique loan.
use super::*;
impl<Schema: worth_query_installation::facade::ApplicationSchema, Operation, Input, Scope>
    CheckedMutableSourceAdmission<'_, Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation) fn consume_pending<
        Query,
    >(
        self,
        pending: WorthQueryPendingSourceExpectation<Query>,
        phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        let Self { runtime, admission } = self;

        if pending.admission_identity != admission.admission_identity()
            || !admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| {
                    pending
                        .selected_product
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
                pending.expected_query_identifier,
            )
        })?;
        let query_identity = runtime
            .installed_schema
            .installed_query_identity_by_name(pending.expected_query_identifier)
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                    pending.expected_query_identifier,
                )
            })?;
        pending.source.validate_affinity(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            admission.graph_work_branch(),
            admission.scope_entity_id(),
            &pending.selected_product,
            pending.expected_query_identifier,
            query_identity,
        )?;
        // The pending owner carries only source custody across resolution. Borrow
        // this advancement now, after idempotency selected fresh execution.
        let control = SourceFactExpansionControl::for_phase(
            runtime,
            phase,
            Some(admission.publication_request()),
            pending.expected_query_identifier,
        )?;
        let facts = pending.source.validated_facts(
            &graph.layout,
            pending.expected_query_identifier,
            control,
        )?;
        admission.bind_source_facts(facts, pending.source.retain_expression_binding());
        Ok(())
    }
}
