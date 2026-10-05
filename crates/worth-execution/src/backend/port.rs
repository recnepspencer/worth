use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Mutex,
};

use worth_foundational::{
    ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture, ExecutionReport,
    PartitionIdentity,
};

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    report::ChargedBytes,
};

use super::{
    admission::AdmittedBatch,
    meter::{record_nested, KernelContext, KernelFailure, RunLimits},
    native, perturbation, serial, settlement, PreparedBatchResources,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BackendKind {
    Serial,
    Native,
    Perturbation(u64),
}

pub(crate) struct TaskOutcome<R, E> {
    pub(crate) result: Result<R, KernelFailure<E>>,
    pub(crate) work: u64,
    pub(crate) span: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BatchStop<E> {
    Failure {
        identity: PartitionIdentity,
        cause: KernelFailure<E>,
    },
    WorkExhausted {
        identity: PartitionIdentity,
    },
    Admission(LeaseDenial),
}

pub(crate) struct BatchOutcome<R, E> {
    pub(crate) values: Vec<R>,
    pub(crate) stop: Option<BatchStop<E>>,
    /// Exclusive identity boundary; every earlier partition settled.
    pub(crate) prefix_boundary: Option<PartitionIdentity>,
    pub(crate) report: ExecutionReport,
}

pub(crate) fn run_checked_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_with_charge(lease, batch, backend, kernel, true)
}

pub(crate) fn run_checked_batch_with_charge<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_inner(lease, batch, backend, kernel, charge_parent, None)
}

pub(crate) fn run_checked_batch_prepared<T, R, E, F>(
    lease: &ExecutionResourceLease<'_>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    resources: PreparedBatchResources,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_inner(Some(lease), batch, backend, kernel, true, Some(resources))
}

fn run_checked_batch_inner<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
    prepared: Option<PreparedBatchResources>,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    if lease.is_none() && RunLimits::has_leased_parent() {
        let outcome = denied(LeaseDenial::UnrelatedNestedLease);
        record_nested(outcome.report, true);
        return outcome;
    }
    if prepared
        .as_ref()
        .is_some_and(|resources| !resources.matches_current_parent())
    {
        let outcome = denied(LeaseDenial::UnrelatedNestedLease);
        if charge_parent {
            record_nested(outcome.report, true);
        }
        return outcome;
    }
    let limits = if let Some(resources) = prepared.as_ref() {
        resources.dispatch_limits(lease.expect("prepared map requires its lease"), backend)
    } else {
        RunLimits::for_run(lease, matches!(backend, BackendKind::Serial))
    };
    let requested_posture = if lease
        .is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic)
        && !limits.force_serial()
        && !matches!(backend, BackendKind::Serial)
    {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let initial_fallback = if requested_posture == ExecutionPosture::Serial {
        Some(match lease {
            None => ExecutionFallbackCause::NoLease,
            Some(_) if cfg!(target_arch = "wasm32") => ExecutionFallbackCause::PlatformSerial,
            Some(value) if value.policy().posture() == ExecutionPosture::Serial => {
                ExecutionFallbackCause::PolicySerial
            }
            Some(_) if matches!(backend, BackendKind::Serial) => {
                ExecutionFallbackCause::OracleSerial
            }
            Some(_) => ExecutionFallbackCause::WorkerLimit,
        })
    } else {
        None
    };
    if batch.len() == 0 {
        let physical = if prepared.is_some() {
            ExecutionPhysicalReport::new(
                limits.worker_high_watermark(),
                limits.peak_memory(),
                Some(0),
                0,
                limits.discarded_work(),
            )
        } else {
            ExecutionPhysicalReport::default()
        };
        let outcome = BatchOutcome {
            values: Vec::new(),
            stop: None,
            prefix_boundary: None,
            report: ExecutionReport::new(ExecutionPosture::Serial, 0, 0, physical)
                .with_fallback(ExecutionFallbackCause::NoWork),
        };
        if charge_parent {
            record_nested(outcome.report, false);
        }
        return outcome;
    }
    let max_workers = lease.map_or(0, |value| value.policy().budget().max_workers().get());
    let memory_bytes = batch.execution_memory_bytes::<R, E>().and_then(|bytes| {
        bytes.checked_add(limits.framework_context_bytes(batch.len(), max_workers)?)
    });
    let Some(memory_bytes) = memory_bytes else {
        let outcome = denied(LeaseDenial::ResourceExhausted);
        if charge_parent {
            record_nested(outcome.report, true);
        }
        return outcome;
    };
    if prepared
        .as_ref()
        .is_some_and(|resources| !resources.matches_dispatch(memory_bytes))
    {
        let outcome = denied(LeaseDenial::UnrelatedNestedLease);
        if charge_parent {
            record_nested(outcome.report, true);
        }
        return outcome;
    }
    let reservation = if let Some(lease) = lease {
        let admission_bytes = if prepared.is_some() { 0 } else { memory_bytes };
        match lease.reserve_entry(admission_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => {
                let outcome = denied(denial);
                if charge_parent {
                    record_nested(outcome.report, true);
                }
                return outcome;
            }
        }
    } else {
        None
    };
    let _memory_activity = if prepared.is_some() {
        None
    } else {
        reservation
            .as_ref()
            .map(|_| limits.enter_memory(memory_bytes))
    };
    // The reservation remains alive through settlement and result-vector
    // allocation; the returned values leave the lease at this boundary.
    let outcomes: Vec<Mutex<Option<TaskOutcome<R, E>>>> =
        (0..batch.len()).map(|_| Mutex::new(None)).collect();
    let execute = |index: usize| {
        let newly_active_worker = lease.is_some_and(|value| !value.is_current_worker_in_lineage());
        let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
        let mut context = KernelContext::new(limits.clone());
        let result = match limits.safe_point() {
            Err(stop) => Err(KernelFailure::Stop(stop)),
            Ok(()) => {
                let _physical_worker = limits.enter_physical_worker(newly_active_worker);
                let result = {
                    let _meter_context = context.enter();
                    catch_unwind(AssertUnwindSafe(|| {
                        kernel(batch.value(index), &mut context)
                    }))
                    .unwrap_or(Err(KernelFailure::Panic))
                };
                result
            }
        };
        let result = if let Some(stop) = context.checkpoint_stop() {
            Err(KernelFailure::Stop(stop))
        } else if context.nested_stopped() && result.is_ok() {
            Err(KernelFailure::Stop(super::meter::KernelStop::NestedStopped))
        } else {
            result
        };
        let result = match result {
            Ok(value) if value.additional_charged_bytes() > batch.result_capacity(index) => {
                Err(KernelFailure::ResultCapacityExceeded)
            }
            Err(KernelFailure::Domain(error))
                if error.additional_charged_bytes() > batch.result_capacity(index) =>
            {
                Err(KernelFailure::ResultCapacityExceeded)
            }
            other => other,
        };
        let (work, span) = context.cost();
        let mut slot = outcomes[index]
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *slot = Some(TaskOutcome { result, work, span });
    };
    let admitted_workers = match (requested_posture, backend) {
        (ExecutionPosture::Serial, _) | (_, BackendKind::Serial) => {
            serial::run(batch.len(), &execute);
            1
        }
        (_, BackendKind::Native) => {
            native::run(lease.expect("native requires lease"), batch.len(), &execute)
        }
        (_, BackendKind::Perturbation(seed)) => perturbation::run(
            lease.expect("perturbation requires lease"),
            batch.len(),
            seed,
            &execute,
        ),
    };
    let resolved_posture = if admitted_workers > 1 {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let fallback = if requested_posture == ExecutionPosture::Automatic && admitted_workers == 1 {
        Some(ExecutionFallbackCause::Capacity)
    } else {
        initial_fallback
    };
    let active_high_water = limits.worker_high_watermark();
    let queue_width = if !matches!(backend, BackendKind::Serial)
        && requested_posture == ExecutionPosture::Automatic
    {
        batch.len()
    } else {
        0
    };
    let outcome = settlement::settle(
        batch,
        outcomes,
        &limits,
        resolved_posture,
        active_high_water,
        limits.peak_memory(),
        queue_width,
        if matches!(backend, BackendKind::Serial) || requested_posture == ExecutionPosture::Serial {
            Some(0)
        } else {
            None
        },
        fallback,
    );
    if charge_parent {
        record_nested(outcome.report, outcome.stop.is_some());
    }
    drop(reservation);
    outcome
}

fn denied<R, E>(denial: LeaseDenial) -> BatchOutcome<R, E> {
    BatchOutcome {
        values: Vec::new(),
        stop: Some(BatchStop::Admission(denial)),
        prefix_boundary: None,
        report: ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        )
        .with_fallback(ExecutionFallbackCause::Capacity),
    }
}
