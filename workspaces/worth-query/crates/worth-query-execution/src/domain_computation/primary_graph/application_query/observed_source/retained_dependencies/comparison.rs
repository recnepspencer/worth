//! One observation, one execution and one cumulative native comparison meter.
use super::*;
pub(super) mod observation;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StorageControl,
    output_reuse::{first_moved_fact, retained_fact_denial},
    request_allocation_control::RequestAllocationControl,
    WorthQueryOutputDemandSettlement,
};
use std::num::NonZeroUsize;
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
        maximum_work: NonZeroUsize,
    ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
        let runtime = self.application();
        let execution = phase
            .checked_execution_request_for(&runtime.product_runtime)
            .map_err(execution_denial)?;
        let control = RequestAllocationControl::new(request, execution.allocation_policy());
        let mut admission = self.request_admission(maximum_work);
        let outcome = runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |relational| {
                self.require_current_settlements_in(
                    execution,
                    relational,
                    settlements,
                    &mut admission,
                )?;
                for source in dependencies.sources.elements() {
                    self.require_current_dependency_in(
                        &execution,
                        relational,
                        source,
                        control.policy(),
                        request,
                        &mut admission,
                    )?;
                }
                Ok(())
            });
        observation::record(runtime, admission.charged_work());
        outcome
    }
    // Single-member comparison is owner-local. It issues no complete-set proof;
    // production's loop lends the SAME admission to every member.
    pub(super) fn require_current_dependency_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        source: &WorthQueryObservedSource<()>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        request: &WorthQueryRequestScope,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<(), WorthQueryRequiredSetCurrentnessDenial> {
        let runtime = self.application();
        policy.check_live().map_err(storage)?;
        admit_descriptor(admission, source)?;
        self.validate_dependency(source, false)?;
        let prepared = source.admit_fact_materialization(
            &runtime.primary_provider.graph.layout,
            admission,
            &source.query_identifier,
        )?;
        let facts = prepared.into_facts(StorageControl::new(policy, Some(request)))?;
        match first_moved_fact(
            execution,
            relational,
            self.application_basis().snapshot_handle(),
            facts.iter(),
            admission,
        ) {
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
        Ok(())
    }
}
