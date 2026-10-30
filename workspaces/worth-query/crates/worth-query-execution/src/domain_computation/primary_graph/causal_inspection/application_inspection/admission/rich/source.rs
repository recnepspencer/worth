//! One actual source diagnostic loan assembles only requested original source effects.
use super::super::super::{
    report::rich::FundedRichInspectionReport,
    rich::{source::SourceDetail, OriginalQueryDetail},
};
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::run_retained_stage;
impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in super::super::super) fn materialize_source_rich<Q, P, R, S>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<Schema, Q, P, R, S>,
    ) -> Result<FundedRichInspectionReport, InspectionAdmissionDenial> {
        drop(self.check(execution)?);
        let diagnostic = self
            .diagnostic
            .as_ref()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let loan = run_retained_stage(execution, self.memory, |context, _| {
            diagnostic.admit_diagnostic_comparison(
                context,
                query,
                &self.session,
                &self.target,
                &self.target.parameters(),
                self.principal,
                self.scope.entity_id(),
            )
        })
        .map_err(InspectionAdmissionDenial::DiagnosticAdmission)?;
        let request = self
            .target
            .rich_request()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let original = self
            .target
            .source()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let source = if request.requests_source_effects() {
            super::super::super::rich::source::capture(
                execution,
                self.memory,
                original,
                self.scope.entity_kind(),
                loan,
                &self.runtime.primary_provider.graph.layout,
                self.runtime.installed_schema().native_contracts(),
            )?
        } else {
            SourceDetail::NotRequested
        };
        let prepared = matches!(&source, SourceDetail::Provided(_));
        let body = PreparedRichMaterialization {
            scope: RichReportScope {
                session: self.session.identity(),
                plan: self.session.plan_identity(),
                target: Arc::clone(&self.target),
                request,
            },
            artifact: WorthQueryApplicationRichMaterialization {
                request,
                source,
                producer: None,
                original: if request.requests(
                    super::super::super::super::inventory::CausalEvidenceFamily::QueryInspection,
                ) {
                    OriginalQueryDetail::Unselected
                } else {
                    OriginalQueryDetail::NotRequested
                },
            },
        };
        if prepared {
            self.runtime
                .primary_provider
                .inspection_materialization_prepared();
        }
        drop(self.check(execution)?);
        let reference = self.materialize(execution)?;
        let report = super::super::super::report::rich::materialize_rich(
            execution,
            reference,
            body,
            self.memory,
        )
        .map_err(InspectionAdmissionDenial::Report)?;
        drop(self.check(execution)?);
        Ok(report)
    }
}
