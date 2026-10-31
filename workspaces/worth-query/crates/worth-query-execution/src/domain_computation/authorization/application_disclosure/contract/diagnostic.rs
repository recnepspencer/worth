//! Diagnostic use compiles the real field contract under its original payer.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};

mod allocation;
mod work;

pub(in crate::domain_computation) enum DiagnosticContractDenial {
    NotDeclared,
    UnsupportedShape,
    Storage(RetainedStorageDenial),
    Contract(WorthQueryApplicationDisclosureContractDenial),
}

/// Public payload visibility supplies no diagnostic field contract. This check
/// precedes any lookup of an original retained projection or its availability.
pub(in crate::domain_computation) fn compile_diagnostic_disclosure_contract<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryAdmittedApplicationDisclosureContract, CaptureStop<DiagnosticContractDenial>>
{
    run_retained_stage(execution, memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if query.disclosure().posture() != ApplicationQueryDisclosurePosture::Governed {
            return Err(MapKernelFailure::Domain(
                DiagnosticContractDenial::NotDeclared,
            ));
        }
        if !diagnostic_shape_supported(query) {
            return Err(MapKernelFailure::Domain(
                DiagnosticContractDenial::UnsupportedShape,
            ));
        }
        let bytes = allocation::backing_bytes(query, context)?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory.resize(bytes).map_err(|cause| {
                MapKernelFailure::Domain(DiagnosticContractDenial::Storage(cause))
            })?;
        }
        super::compilation::compile_with_admission(query, layout, &mut |operation| {
            work::admit(context, operation)
        })
        .map_err(|cause| match cause {
            super::admission::CompilationStop::Admission(cause) => cause,
            super::admission::CompilationStop::Contract(cause) => {
                MapKernelFailure::Domain(DiagnosticContractDenial::Contract(cause))
            }
        })
    })
}

pub(in crate::domain_computation::authorization::application_disclosure) fn diagnostic_shape_supported<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
) -> bool {
    let graph = query.read_graph();
    graph.root_paths().is_empty()
        && graph.orbit().is_none()
        && graph.component_selection().is_none()
        && query.continuation().is_none()
        && query.live().is_none()
}
