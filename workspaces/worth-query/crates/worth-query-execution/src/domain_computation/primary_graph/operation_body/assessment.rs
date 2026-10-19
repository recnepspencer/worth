//! Read-only assessment borrows resources from the exact native runtime phase.
use super::WorthQueryOperationBodyRequirementFinding;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryForeignAdvancementPhase,
    WorthQueryOperationProjectionDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationMutationBinding,
};

/// Descriptive findings retain their native memory claim for their whole lifetime.
/// This result carries no projection snapshot or mutation authority.
pub struct WorthQueryCurrentBodyRequirementAssessment {
    findings: Vec<WorthQueryOperationBodyRequirementFinding>,
    _memory: Option<
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    >,
}

impl WorthQueryCurrentBodyRequirementAssessment {
    pub fn findings(&self) -> &[WorthQueryOperationBodyRequirementFinding] {
        &self.findings
    }
    pub(in crate::domain_computation::primary_graph::operation_body) fn new(
        findings: Vec<WorthQueryOperationBodyRequirementFinding>,
        memory: crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    ) -> Self {
        Self {
            findings,
            _memory: Some(memory),
        }
    }
    pub(in crate::domain_computation::primary_graph::operation_body) fn empty() -> Self {
        Self {
            findings: Vec::new(),
            _memory: None,
        }
    }
}

/// A native body assessment preserves phase, projection and observation refusal.
#[derive(Debug)]
pub enum WorthQueryOperationBodyAssessmentDenial {
    Source(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Phase(WorthQueryForeignAdvancementPhase),
    Projection(WorthQueryOperationProjectionDenial),
    Observation(HandlerExecutionDenial),
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Inspects the installed body's requirements using an already authorized
    /// operation. The result retains its owned bytes; no snapshot or mutation target escapes.
    pub fn assess_admitted_operation_body_requirements<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let execution = phase
            .execution_for(&self.product_runtime)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Phase)?;
        let request = phase
            .execution_request_for(&self.product_runtime)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Phase)?;
        self.validate_projected_source_facts(&request, admission)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Source)?;
        let projected = projection
            .project_admitted_operation(
                phase,
                admission,
                |reader, identity| reader.inspect_body_requirements(identity, installed, execution),
                allocation,
            )
            .map_err(WorthQueryOperationBodyAssessmentDenial::Projection)?;
        let (findings, snapshot, _) = projected.into_parts();
        drop(snapshot);
        findings.map_err(WorthQueryOperationBodyAssessmentDenial::Observation)
    }
}

#[cfg(test)]
#[path = "assessment/tests.rs"]
mod tests;
