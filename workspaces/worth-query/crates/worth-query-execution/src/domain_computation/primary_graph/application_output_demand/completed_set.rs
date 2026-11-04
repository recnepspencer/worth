//! One Native qualification path for complete original output sets.
use super::WorthQueryOutputDemandSettlement;
use crate::publication_boundary::{
    discovery_currentness::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    },
    WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind,
    crate::domain_computation::primary_graph::WorthQueryAdvancementPhase,
    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation,
};

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub(super) fn qualify_completed_output_set<'s>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
        retained_bytes: u64,
    ) -> Result<
        (
            Arc<WorthQueryApplicationReadObservation>,
            Option<worth_execution::ExecutionMemoryReservation>,
        ),
        WorthQueryRequiredSetCurrentnessDenial,
    > {
        let refused =
            |kind| WorthQueryOutputDemandDenial::new(kind, "completed output-set qualification");
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(|cause| refused(Kind::ExecutionRequest(cause)))?;
        let memory = execution.reserve_retained_memory(retained_bytes).map_err(|cause| {
            refused(Kind::ExecutionRequest(
                crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause),
            ))
        })?;
        let settlements = std::iter::once(root).chain(dependents);
        match dependencies {
            Some(dependencies) => {
                self.require_current_required_set(phase, access, scope, settlements, dependencies)?
            }
            None => self.require_current_output_settlements(phase, settlements)?,
        }
        execution.check_live().map_err(|cause| {
            refused(Kind::ExecutionRequest(
                crate::domain_computation::primary_graph::application_contribution::advancement_denial(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                ),
            ))
        })?;
        Ok((self.retain_application_read(), memory))
    }
}
