//! The fresh diagnostic loan precedes every comparison of original payload facts.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::{checkpoint, run_retained_stage},
    application_query::compare_original_projections,
    causal_inspection::CausalInspectionReason,
};
use worth_execution::MapKernelFailure;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationQuery};

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn compare<
        Query,
        Parameters,
        QueryResult,
        QueryScope,
    >(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        query: &WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            QueryScope,
        >,
        source: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        >,
    ) -> Result<
        (
            CausalInspectionReason,
            Option<super::super::connection::WorthQueryApplicationProducerConnection>,
        ),
        InspectionAdmissionDenial,
    > {
        drop(self.check(execution)?);
        let diagnostic = self
            .diagnostic
            .as_ref()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let admission = run_retained_stage(execution, self.memory, |context, _| {
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
        // Both real capabilities and the current READ have now passed on the
        // same session. The original metadata gates included all six facets.
        let before = self
            .target
            .before()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let after = self
            .target
            .original()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        run_retained_stage(execution, self.memory, |context, _| {
            let work = u64::try_from(std::mem::size_of::<
                [worth_foundational::facade::CanonicalDigestId; 2],
            >())
            .ok()
            .and_then(|bytes| bytes.checked_add(2))
            .ok_or(MapKernelFailure::Stop(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            if before.source.parameter_binding_identity != after.source.parameter_binding_identity {
                return Err(MapKernelFailure::Domain(()));
            }
            Ok(())
        })
        .map_err(InspectionAdmissionDenial::ComparisonParameters)?;
        require_request(execution, self.request)?;
        let before = before
            .source
            .original_projection()
            .ok_or(InspectionAdmissionDenial::MissingOriginalProjection)?;
        let after = after
            .source
            .original_projection()
            .ok_or(InspectionAdmissionDenial::MissingOriginalProjection)?;
        let equal = compare_original_projections(execution, self.memory, before, after, &admission)
            .map_err(InspectionAdmissionDenial::Comparison)?;
        require_request(execution, self.request)?;
        let connection = source
            .map(|source| {
                super::super::connection::connect(
                    self.runtime,
                    execution,
                    &self.target,
                    source,
                    &admission,
                    None,
                    self.memory,
                )
            })
            .transpose()
            .map_err(InspectionAdmissionDenial::Connection)?;
        Ok((
            if equal {
                CausalInspectionReason::UnchangedResult
            } else {
                CausalInspectionReason::ChangedResult
            },
            connection,
        ))
    }
}
