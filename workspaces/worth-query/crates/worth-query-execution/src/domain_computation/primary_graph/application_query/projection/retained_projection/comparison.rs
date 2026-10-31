//! Compare genuine original disclosed trees only after fresh diagnostic admission.
use super::OriginalProjection;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    primary_graph::admitted_capture::{
        checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
    },
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};

pub(in crate::domain_computation::primary_graph) enum ProjectionComparisonDenial {
    DisclosureInterpretationChanged,
    OriginalDisclosureIncomplete,
    NestingExceedsDeclaredShape,
    Storage(RetainedStorageDenial),
    Allocation(std::collections::TryReserveError),
}

/// True means the complete originally disclosed projections are equal. Source
/// identities, predicate provenance and field revisions do not enter equality.
pub(in crate::domain_computation::primary_graph) fn compare_original_projections(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    before: &OriginalProjection,
    after: &OriginalProjection,
    admission: &DiagnosticComparisonAdmission<'_>,
) -> Result<bool, CaptureStop<ProjectionComparisonDenial>> {
    run_retained_stage(execution, memory, |context, mut memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let graph = admission.graph();
        let expected = graph
            .projections()
            .len()
            .checked_add(graph.relations().len())
            .ok_or(MapKernelFailure::Domain(
                ProjectionComparisonDenial::Storage(RetainedStorageDenial::Overflow),
            ))?;
        if before.disclosure.len() != expected || after.disclosure.len() != expected {
            return Err(MapKernelFailure::Domain(
                ProjectionComparisonDenial::DisclosureInterpretationChanged,
            ));
        }
        for ((left, right), expected) in before.disclosure.iter().zip(after.disclosure.iter()).zip(
            graph
                .projections()
                .iter()
                .map(|field| field.slot_key_identity())
                .chain(
                    graph
                        .relations()
                        .iter()
                        .map(|relation| relation.slot_key_identity()),
                ),
        ) {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            if !Arc::ptr_eq(&left.0, &expected)
                || !Arc::ptr_eq(&right.0, &expected)
                || left.1 != right.1
            {
                return Err(MapKernelFailure::Domain(
                    ProjectionComparisonDenial::DisclosureInterpretationChanged,
                ));
            }
            // This narrow result promises complete query-output equality. A
            // partial original cannot establish that promise, even when all
            // currently visible fields happen to compare equal.
            if !left.1 {
                return Err(MapKernelFailure::Domain(
                    ProjectionComparisonDenial::OriginalDisclosureIncomplete,
                ));
            }
        }
        let depth = graph
            .relations()
            .len()
            .checked_add(1)
            .ok_or(MapKernelFailure::Domain(
                ProjectionComparisonDenial::Storage(RetainedStorageDenial::Overflow),
            ))?;
        super::super::projected_tree::comparison::rows_equal(
            &before.rows,
            &after.rows,
            before.disclosure.elements(),
            depth,
            context,
            |bytes| {
                if let Some(memory) = memory.as_deref_mut() {
                    memory.resize(bytes).map_err(|cause| {
                        MapKernelFailure::Domain(ProjectionComparisonDenial::Storage(cause))
                    })?;
                }
                Ok(())
            },
        )
    })
}
