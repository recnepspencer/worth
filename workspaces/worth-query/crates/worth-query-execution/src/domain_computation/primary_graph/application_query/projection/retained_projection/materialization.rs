//! The original projection owner consumes a fresh diagnostic loan before issuing a view.
pub use super::super::projected_tree::materialization::{
    WorthQueryMaterializedFieldValue, WorthQueryMaterializedQueryField,
    WorthQueryMaterializedQueryRelation, WorthQueryMaterializedQueryRow,
};
use super::OriginalProjection;
use crate::domain_computation::{
    authorization::application_disclosure::DiagnosticComparisonAdmission,
    primary_graph::admitted_capture::{checkpoint, run_retained_stage, CaptureStop},
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};

/// A shared descriptive view. The original Arc keeps its original payer alive.
pub struct WorthQueryMaterializedQueryProjection {
    original: Arc<OriginalProjection>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProjectionMaterializationRefusal {
    DisclosureInterpretationChanged,
    OriginalDisclosureIncomplete,
}
impl WorthQueryMaterializedQueryProjection {
    pub fn rows(&self) -> impl ExactSizeIterator<Item = WorthQueryMaterializedQueryRow<'_>> {
        super::super::projected_tree::materialization::rows(&self.original.rows)
    }
}
pub(in crate::domain_computation::primary_graph) fn materialize_original_projection(
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
    original: &Arc<OriginalProjection>,
    admission: DiagnosticComparisonAdmission<'_>,
) -> Result<
    WorthQueryMaterializedQueryProjection,
    CaptureStop<WorthQueryProjectionMaterializationRefusal>,
> {
    use WorthQueryProjectionMaterializationRefusal as Denial;
    run_retained_stage(execution, memory, |context, _| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let graph = admission.graph();
        let expected = graph
            .projections()
            .len()
            .checked_add(graph.relations().len())
            .ok_or(MapKernelFailure::Stop(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
        if original.disclosure.len() != expected {
            return Err(MapKernelFailure::Domain(
                Denial::DisclosureInterpretationChanged,
            ));
        }
        for ((slot, disclosed), expected) in original.disclosure.iter().zip(
            graph
                .projections()
                .iter()
                .map(|f| f.slot_key_identity())
                .chain(graph.relations().iter().map(|r| r.slot_key_identity())),
        ) {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            if !Arc::ptr_eq(slot, &expected) {
                return Err(MapKernelFailure::Domain(
                    Denial::DisclosureInterpretationChanged,
                ));
            }
            if !disclosed {
                return Err(MapKernelFailure::Domain(
                    Denial::OriginalDisclosureIncomplete,
                ));
            }
        }
        Ok(WorthQueryMaterializedQueryProjection {
            original: Arc::clone(original),
        })
    })
}
