use super::super::types::ExecutionReport;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationProgress};
use crate::data::request_preparation::SignalPreparationBudget;
use worth_execution::{
    ExecutionResourceLease, ExecutionScan, MapKernelContext, MapKernelFailure, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

/// One ordered request boundary owns planning, stage epochs, and publication.
/// Its report is the authority's actual cumulative measurement.
pub(crate) fn run_signal_request_scope(
    lease: &ExecutionResourceLease<'_>,
    mut execute: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut SignalPublicationProgress,
        &mut SignalPreparationBudget,
    ) -> Result<ExecutionReport, SignalError>,
) -> Result<ExecutionReport, SignalError> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).map_err(
        |denial| SignalError::internal(format!("signal request admission failed: {denial:?}")),
    )?;
    let memory = lease.policy().budget().charged_memory_bytes();
    let mut complete = None;
    let mut progress = SignalPublicationProgress::default();
    let mut preparation = SignalPreparationBudget::new(memory / 8);
    let outcome = scan.run(
        Some(lease),
        (),
        0,
        0,
        memory / 8,
        memory / 8,
        |_, _, request_work| {
            request_work.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            // Retain the enclosing physical report before any graph effects.
            preparation
                .claim_retained_vec::<worth_foundational::ExecutionReport>(1)
                .map_err(MapKernelFailure::Domain)?;
            let mut result = execute(request_work, &mut progress, &mut preparation)
                .map_err(MapKernelFailure::Domain)?;
            if result.execution.len() == result.execution.capacity() {
                result.execution.reserve_exact(1);
            }
            complete = Some(result);
            Ok(((), ()))
        },
    );
    match outcome {
        ScanOutcome::Complete {
            report: physical, ..
        } => {
            let mut report = complete.expect("completed Signal request scan has a report");
            report.execution.push(physical);
            Ok(report)
        }
        ScanOutcome::Stopped {
            reason,
            boundary,
            report,
            ..
        } => Err(SignalError::execution_stopped(SignalExecutionStop::new(
            reason.into(),
            boundary,
            progress,
            report,
        ))),
    }
}
