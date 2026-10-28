//! Validate retained outputs and original query dependencies at one selected image.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::output_reuse::retained_fact_denial;
use crate::domain_computation::primary_graph::output_reuse::{
    first_moved_fact_in, RetainedFactVerification,
};
use crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Validates every retained output and every original query dependency at
    /// this selected image. An empty output iterator does not omit discovery.
    pub fn require_current_required_set<'s>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        request: &WorthQueryRequestScope,
        settlements: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        dependencies: &WorthQueryRetainedQueryDependencies,
    ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
        let runtime = self.application();
        let execution = phase
            .checked_execution_request_for(&runtime.product_runtime)
            .map_err(execution_denial)?;
        let control = RequestAllocationControl::new(request, execution.allocation_policy());

        runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                let query_control = || match request.interruption() {
                    None => Ok(()),
                    Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled) => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled),
                    Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded) => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed),
                };
                let mut verification = RetainedFactVerification::new(&execution, relational, self.application_basis().snapshot_handle(), Some(&query_control));
                self.require_current_settlements_in_extent(execution, relational, settlements, &mut verification)?;
                for source in dependencies.sources.elements() {
                    self.require_current_dependency_in(
                        relational,
                        source,
                        control.policy(),
                        request,
                        &mut verification,
                    )?;
                }
                Ok(())
            })
    }
    // Single-member comparison is owner-local. It issues no complete-set proof;
    // production validates each member at the same selected native image.
    pub(super) fn require_current_dependency_in(
        &self,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        source: &super::RetainedQuerySource,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        request: &WorthQueryRequestScope,
        verification: &mut RetainedFactVerification<'_, '_, '_>,
    ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
        let runtime = self.application();
        policy.check_live().map_err(storage)?;

        self.validate_dependency(&source.source, false)?;
        let selected = if source.coverage.is_some() {
            super::coverage::selected_basis(relational, self.application_basis().snapshot_handle())?
        } else {
            None
        };
        if let (Some(coverage), Some(selected)) = (&source.coverage, &selected)
            && runtime
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner
                .query_facts_clean_at(coverage, selected, policy)
                .map_err(super::coverage::coverage_stop)?
                .is_some()
        {
            return Ok(());
        }
        let retained = source;
        let source = &retained.source;
        let prepared = source.admit_fact_materialization(
            &runtime.primary_provider.graph.layout,
            &source.query_identifier,
        )?;
        let facts = prepared.into_facts(StorageControl::new(policy, Some(request)))?;
        match first_moved_fact_in(verification, facts.iter()) {
            Ok(None) => {}
            Ok(Some(fact)) => {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    format!("original discovery fact moved: {}", fact.locator_identity()),
                )
                .into())
            }
            Err(stop) => {
                return Err(
                    retained_fact_denial(stop, "original discovery fact unavailable").into(),
                )
            }
        }
        if let (Some(coverage), Some(selected)) = (&retained.coverage, &selected) {
            runtime
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner
                .reestablish_query_read(coverage, selected, policy)
                .map_err(super::coverage::coverage_stop)?;
        }
        policy.check_live().map_err(storage)?;
        Ok(())
    }
}
