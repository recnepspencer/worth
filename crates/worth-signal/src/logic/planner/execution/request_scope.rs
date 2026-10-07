use super::super::types::ExecutionReport;
use crate::data::error::{SignalError, SignalPublicationProgress};
use crate::data::request_preparation::SignalPreparationBudget;
use worth_execution::{
    ExecutionResourceLease, ExecutionScan, MapKernelContext, MapKernelFailure, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

/// One ordered request boundary owns planning, stage epochs, and publication.
/// Its report is the authority's actual cumulative measurement.
pub(crate) fn run_signal_request_scope(
    lease: &ExecutionResourceLease<'_>,
    execute: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut SignalPublicationProgress,
        &mut SignalPreparationBudget,
    ) -> Result<ExecutionReport, SignalError>,
) -> Result<ExecutionReport, SignalError> {
    run_signal_execution_request_scope(worth_execution::ExecutionRequest::leased(lease), execute)
}

pub(crate) fn run_signal_execution_request_scope(
    request: worth_execution::ExecutionRequest<'_, '_>,
    mut execute: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut SignalPublicationProgress,
        &mut SignalPreparationBudget,
    ) -> Result<ExecutionReport, SignalError>,
) -> Result<ExecutionReport, SignalError> {
    let (mut report, physical) =
        run_signal_preparation_request(request, |work, progress, preparation| {
            let mut report = execute(work, progress, preparation)?;
            if report.execution.len() == report.execution.capacity() {
                report.execution.reserve_exact(1);
            }
            Ok(report)
        })?;
    report.execution.push(physical);
    Ok(report)
}

/// Owns preparation of either an execution result or a materialized plan.
/// The operation admits its retained shape on the required preparation owner.
pub(crate) fn run_signal_preparation_request<R>(
    request: worth_execution::ExecutionRequest<'_, '_>,
    mut prepare: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut SignalPublicationProgress,
        &mut SignalPreparationBudget,
    ) -> Result<R, SignalError>,
) -> Result<(R, worth_foundational::ExecutionReport), SignalError> {
    request
        .in_scope(|lease| run_signal_request_in_scope(request.memory_limit(), lease, &mut prepare))
        .map_err(SignalError::execution_scope_denied)?
}

fn run_signal_request_in_scope<R>(
    memory: u64,
    lease: Option<&ExecutionResourceLease<'_>>,
    execute: &mut impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut SignalPublicationProgress,
        &mut SignalPreparationBudget,
    ) -> Result<R, SignalError>,
) -> Result<(R, worth_foundational::ExecutionReport), SignalError> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).map_err(
        |denial| SignalError::internal(format!("signal request admission failed: {denial:?}")),
    )?;
    let mut complete = None;
    let mut progress = SignalPublicationProgress::default();
    let mut preparation = SignalPreparationBudget::for_request(memory, lease)?;
    let outcome = scan.run(lease, (), 0, 0, 0, 0, |_, _, request_work| {
        request_work.checkpoint(0).map_err(MapKernelFailure::Stop)?;
        // Retain the enclosing physical report before any graph effects.
        preparation
            .claim_retained_vec::<worth_foundational::ExecutionReport>(1)
            .map_err(MapKernelFailure::Domain)?;
        // The request result is carried by its owner, not a scan output with
        // a zero heap grant. Otherwise a boxed stop or domain error is replaced
        // by ResultCapacityExceeded before reaching Signal's conversion door.
        complete = Some(execute(request_work, &mut progress, &mut preparation));
        Ok(((), ()))
    });
    let (physical, stopped) = match outcome {
        ScanOutcome::Complete { report, .. } => (report, None),
        ScanOutcome::Stopped {
            reason,
            boundary,
            report,
            ..
        } => (report, Some((reason, boundary))),
    };
    if matches!(complete, Some(Err(_))) {
        let Some(Err(error)) = complete.take() else {
            unreachable!()
        };
        return Err(SignalError::request_scan_failed(
            error, stopped, identity, progress, physical,
        ));
    }
    if let Some((reason, boundary)) = stopped {
        return Err(SignalError::request_scan_stopped(
            reason, boundary, progress, physical,
        ));
    }
    Ok((
        complete.expect("completed Signal request scan has a result")?,
        physical,
    ))
}
