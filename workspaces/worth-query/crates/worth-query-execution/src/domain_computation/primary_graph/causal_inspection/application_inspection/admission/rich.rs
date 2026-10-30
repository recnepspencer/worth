//! One-use application handoff consumes a live scoped diagnostic loan during assembly.
use super::super::{
    report::rich::FundedRichInspectionReport,
    rich::{
        OriginalQueryDetail, WorthQueryApplicationMaterializationRequest,
        WorthQueryApplicationRichMaterialization,
    },
};
use super::*;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    provider_session::WorthQueryGraphWorkSessionIdentity,
};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

pub(in super::super) struct RichReportScope {
    pub(in super::super) session: WorthQueryGraphWorkSessionIdentity,
    pub(in super::super) plan: WorthQueryGraphWorkPlanIdentity,
    pub(in super::super) target: Arc<WorthQueryApplicationInspectionTarget>,
    pub(in super::super) request: WorthQueryApplicationMaterializationRequest,
}
pub(in super::super) struct PreparedRichMaterialization {
    pub(in super::super) scope: RichReportScope,
    pub(in super::super) artifact: WorthQueryApplicationRichMaterialization,
}
mod source;
struct AdmittedRichProjection<'a> {
    scope: RichReportScope,
    diagnostic: DiagnosticComparisonAdmission<'a>,
}
impl AdmittedRichProjection<'_> {
    fn assemble(
        self,
        execution: &ExecutionRequest<'_, '_>,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<PreparedRichMaterialization, InspectionAdmissionDenial> {
        use super::super::super::inventory::CausalEvidenceFamily;
        let original = if !self
            .scope
            .request
            .requests(CausalEvidenceFamily::QueryInspection)
        {
            OriginalQueryDetail::NotRequested
        } else {
            let original = self
                .scope
                .target
                .original()
                .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
            // The actual diagnostic loan exists before even this availability lookup.
            match original.source.original_projection() {
                None=>OriginalQueryDetail::NotRetained,
                Some(projection)=>OriginalQueryDetail::Provided(
                    crate::domain_computation::primary_graph::application_query::materialize_original_projection(
                        execution,memory,projection,self.diagnostic).map_err(InspectionAdmissionDenial::Rich)?),
            }
        };
        Ok(PreparedRichMaterialization {
            artifact: WorthQueryApplicationRichMaterialization {
                request: self.scope.request,
                original,
                source: if self.scope.request.requests_source_effects() {
                    super::super::rich::source::SourceDetail::Unselected
                } else {
                    super::super::rich::source::SourceDetail::NotRequested
                },
                producer: None,
            },
            scope: self.scope,
        })
    }
}
pub(in super::super) fn capture_original_query(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    target: &Arc<WorthQueryApplicationInspectionTarget>,
    session: &WorthQueryManagedGraphWorkSession,
    diagnostic: DiagnosticComparisonAdmission<'_>,
) -> Result<PreparedRichMaterialization, InspectionAdmissionDenial> {
    let request = target
        .rich_request()
        .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
    AdmittedRichProjection {
        diagnostic,
        scope: RichReportScope {
            session: session.identity(),
            plan: session.plan_identity(),
            target: Arc::clone(target),
            request,
        },
    }
    .assemble(execution, memory)
}
impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in super::super) fn materialize_rich<Q, P, R, S>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Q, P, R, S>,
    ) -> Result<FundedRichInspectionReport, InspectionAdmissionDenial> {
        drop(self.check(execution)?);
        let diagnostic = self
            .diagnostic
            .as_ref()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let loan = crate::domain_computation::primary_graph::admitted_capture::run_retained_stage(
            execution,
            self.memory,
            |context, _| {
                diagnostic.admit_diagnostic_comparison(
                    context,
                    query,
                    &self.session,
                    &self.target,
                    &self.target.parameters(),
                    self.principal,
                    self.scope.entity_id(),
                )
            },
        )
        .map_err(InspectionAdmissionDenial::DiagnosticAdmission)?;
        let body =
            capture_original_query(execution, self.memory, &self.target, &self.session, loan)?;
        if matches!(body.artifact.original, OriginalQueryDetail::Provided(_)) {
            self.runtime
                .primary_provider
                .inspection_materialization_prepared();
        }
        drop(self.check(execution)?);
        let reference = self.materialize(execution)?;
        let report =
            super::super::report::rich::materialize_rich(execution, reference, body, self.memory)
                .map_err(InspectionAdmissionDenial::Report)?;
        drop(self.check(execution)?);
        Ok(report)
    }
}

#[cfg(test)]
mod tests;
