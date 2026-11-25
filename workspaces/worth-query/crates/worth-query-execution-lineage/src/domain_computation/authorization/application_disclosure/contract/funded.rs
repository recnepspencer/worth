//! Funded disclosure contract compilation for governed one-shot admission.
//! Every compiler visit, copy, lookup, group intersection, and backing buffer
//! is admitted against the caller's `ExecutionRequest` before allocation.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, MapKernelContext, MapKernelFailure,
    MapKernelStop, ScopeStop, WorkCeilingDenial,
};

mod sizing;

/// Owns a compiled disclosure contract alongside the live memory reservation
/// that pays for its backing. Field declaration order guarantees `contract`
/// always drops BEFORE `memory` on every early-return and error path.
pub struct FundedDisclosureContract {
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    memory: Option<ExecutionMemoryReservation>,
}

impl FundedDisclosureContract {
    pub fn unfunded(contract: WorthQueryAdmittedApplicationDisclosureContract) -> Self {
        Self {
            contract,
            memory: None,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedApplicationDisclosureContract,
        Option<ExecutionMemoryReservation>,
    ) {
        (self.contract, self.memory)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FundedDisclosureCompilationDenial {
    Contract(WorthQueryApplicationDisclosureContractDenial),
    Cancelled,
    DeadlineExceeded,
    WorkExhausted,
    MemoryExhausted,
}

pub(super) enum StageError {
    Contract(WorthQueryApplicationDisclosureContractDenial),
    Memory,
}

pub(super) type Failure = MapKernelFailure<StageError>;

/// Compiles a query's disclosure contract under the caller's `ExecutionRequest`,
/// charging work and reserving temporary/output compilation memory before
/// allocation. Returns `FundedDisclosureContract` so `contract` always drops
/// before its reservation on every return path.
pub fn compile_funded_disclosure_contract<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<FundedDisclosureContract, FundedDisclosureCompilationDenial> {
    let mut memory = None;
    let compiled = run_retained_stage(execution, &mut memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        match query.disclosure().posture() {
            ApplicationQueryDisclosurePosture::Public if query.disclosure().rules().is_empty() => {
                return Ok(WorthQueryAdmittedApplicationDisclosureContract::Public);
            }
            ApplicationQueryDisclosurePosture::Governed => {}
            _ => {
                return Err(MapKernelFailure::Domain(StageError::Contract(denial(
                    query.name(),
                ))));
            }
        }
        let bytes = sizing::backing_bytes(query, context)?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory
                .resize(bytes)
                .map_err(|_: RetainedStorageDenial| MapKernelFailure::Domain(StageError::Memory))?;
        }
        super::compilation::compile_with_admission(query, layout, &mut |operation| {
            admit_work(context, operation)
        })
        .map_err(|cause| match cause {
            super::admission::CompilationStop::Admission(cause) => cause,
            super::admission::CompilationStop::Contract(cause) => {
                MapKernelFailure::Domain(StageError::Contract(cause))
            }
        })
    });
    compiled
        .map(|contract| FundedDisclosureContract { contract, memory })
        .map_err(map_stop)
}

fn map_stop(stop: CaptureStop<StageError>) -> FundedDisclosureCompilationDenial {
    match stop {
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(
            StageError::Contract(cause),
        ))) => FundedDisclosureCompilationDenial::Contract(cause),
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(StageError::Memory)))
        | CaptureStop::Request(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_)))
        | CaptureStop::Kernel(ScopeStop::Admission(LeaseDenial::MemoryExhausted(_))) => {
            FundedDisclosureCompilationDenial::MemoryExhausted
        }
        CaptureStop::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        | CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::Cancelled,
        ))) => FundedDisclosureCompilationDenial::Cancelled,
        CaptureStop::Request(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
        | CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::DeadlineElapsed,
        ))) => FundedDisclosureCompilationDenial::DeadlineExceeded,
        _ => FundedDisclosureCompilationDenial::WorkExhausted,
    }
}

fn admit_work(
    context: &mut MapKernelContext<'_, '_>,
    work: super::admission::CompilationWork<'_>,
) -> Result<(), Failure> {
    let overflow = || MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow);
    let units = match work {
        super::admission::CompilationWork::Visit => 1,
        super::admission::CompilationWork::Copy(bytes) => {
            u64::try_from(bytes).map_err(|_| overflow())?.max(1)
        }
        super::admission::CompilationWork::Scan { entries, parts }
        | super::admission::CompilationWork::Lookup { entries, parts } => {
            let width = parts
                .iter()
                .try_fold(1u64, |sum, text| {
                    sum.checked_add(u64::try_from(text.len()).ok()?.checked_add(1)?)
                })
                .ok_or_else(overflow)?;
            u64::try_from(entries)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(width))
                .ok_or_else(overflow)?
        }
    };
    checkpoint(context, units).map_err(MapKernelFailure::Stop)
}
