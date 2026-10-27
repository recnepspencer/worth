//! Query interruption and acquired identity work surround the original Native read.
use super::super::interruption::NativeReadInterruption;
use super::*;
use crate::domain_computation::primary_graph::aggregate_observation::{
    AggregateObservation, AggregateQuery,
};
use std::sync::Arc;
use worth_relational::facade::runtime::PinnedAggregateReadDenialKind;
pub(super) fn read(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    work: &mut RootSelectionWork<'_>,
    query: AggregateQuery<'_>,
    interruption: &NativeReadInterruption<'_, '_>,
) -> Result<Arc<AggregateObservation>, WorthQueryApplicationReadExecutionDenial> {
    let subject = query.subject;
    work.checkpoint(subject)?;
    let control = || interruption.checkpoint();
    let selected = AggregateObservation::read(
        runtime,
        execution,
        AggregateQuery {
            control: Some(&control),
            ..query
        },
    );
    let units = match &selected {
        Ok(body) => body.native().work_units(),
        Err(stop) => stop.work,
    };
    let charged = charge_identity(work, units, subject);
    if let Some(stop) = interruption.denial() {
        return Err(stop);
    }
    charged?;
    selected.map_err(|stop| match stop.kind {
        PinnedAggregateReadDenialKind::Read(cause) => {
            super::super::denial::relational_read_denial(cause, subject)
        }
        PinnedAggregateReadDenialKind::WorkLimitExceeded => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        ),
        _ => super::path_union::traversal_denial(subject),
    })
}

fn charge_identity(
    work: &mut RootSelectionWork<'_>,
    units: u64,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let units = usize::try_from(units).map_err(|_| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        )
    })?;
    work.work_units = work.work_units.checked_add(units).ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        )
    })?;
    if let Some(spent) = work.spent {
        spent.charge(units, subject)?;
    }
    work.checkpoint(subject)
}
