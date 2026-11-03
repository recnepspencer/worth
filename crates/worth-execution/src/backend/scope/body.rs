//! The typed half of one scope: its kernel and the payload it produced. All of
//! the lifecycle lives in the concrete scope, compiled once; downstream crates
//! instantiate a thin kernel shell per call site and this body per payload type.
use super::admission::{ScopeRequest, ScopeStorage};
use super::execution::run_concrete_scope;
use super::{ScopeHold, ScopeOutcome, ScopeStop};
use crate::authority::{ExecutionResourceLease, SerialRequest};
use crate::backend::meter::{KernelContext, KernelFailure};
use crate::backend::payload_storage::PayloadStorage;
use std::mem::size_of;

/// Stands for the result or domain error the typed body holds. The concrete
/// scope decides what becomes of a payload without ever seeing one.
#[derive(Debug)]
pub(super) struct HeldPayload;

/// How the kernel ended.
pub(super) type KernelVerdict = Result<HeldPayload, KernelFailure<HeldPayload>>;
/// How the scope ended.
pub(super) type ScopeVerdict = Result<HeldPayload, ScopeStop<HeldPayload>>;

/// Everything the concrete scope may ask of a kernel and its payload. Each
/// method runs caller code, so the concrete scope calls them under containment.
pub(super) trait ScopeBody {
    /// Runs the kernel, keeping what it produced in the body.
    fn invoke(
        &mut self,
        context: &mut KernelContext<'_, '_>,
        result_memory: Option<crate::ExecutionResultMemory<'_>>,
    ) -> KernelVerdict;
    /// The heap the held payload charges; nothing for native storage.
    fn payload_bytes(&self) -> u64;
    /// What a held result keeps reserved, or `None` when that overflows.
    fn kept_bytes(&self) -> Option<u64>;
    /// Drops the unrun kernel or the held payload.
    fn discard(&mut self);
}

/// The kernel a scope may still run, erased over its closure so that everything
/// above it instantiates once per payload type rather than once per call site.
trait ScopeKernel<R, E> {
    fn invoke(
        &mut self,
        context: &mut KernelContext<'_, '_>,
        result_memory: Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>>;
    fn discard(&mut self);
}

impl<R, E, F> ScopeKernel<R, E> for Option<F>
where
    F: FnOnce(
        &mut KernelContext<'_, '_>,
        Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>>,
{
    fn invoke(
        &mut self,
        context: &mut KernelContext<'_, '_>,
        result_memory: Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>> {
        let Some(kernel) = self.take() else {
            unreachable!("a scope kernel runs once");
        };
        kernel(context, result_memory)
    }

    fn discard(&mut self) {
        drop(self.take());
    }
}

struct TypedScopeBody<'h, R, E> {
    kernel: &'h mut dyn ScopeKernel<R, E>,
    payload: Option<Result<R, E>>,
    storage: PayloadStorage<(), R, E>,
    kept: Option<&'h dyn Fn(&R) -> Option<u64>>,
}

impl<R, E> ScopeBody for TypedScopeBody<'_, R, E> {
    fn invoke(
        &mut self,
        context: &mut KernelContext<'_, '_>,
        result_memory: Option<crate::ExecutionResultMemory<'_>>,
    ) -> KernelVerdict {
        match self.kernel.invoke(context, result_memory) {
            Ok(value) => {
                self.payload = Some(Ok(value));
                Ok(HeldPayload)
            }
            Err(failure) => {
                let (error, failure) = part_failure(failure);
                if let Some(error) = error {
                    self.payload = Some(Err(error));
                }
                Err(failure)
            }
        }
    }

    fn payload_bytes(&self) -> u64 {
        let PayloadStorage::Metered { result, error, .. } = self.storage else {
            return 0;
        };
        match &self.payload {
            Some(Ok(value)) => result(value),
            Some(Err(failure)) => error(failure),
            None => 0,
        }
    }

    fn kept_bytes(&self) -> Option<u64> {
        let Some(Ok(value)) = &self.payload else {
            return Some(0);
        };
        match self.kept {
            Some(kept) => kept(value),
            None => Some(0),
        }
    }

    fn discard(&mut self) {
        self.kernel.discard();
        drop(self.payload.take());
    }
}

impl<R, E> TypedScopeBody<'_, R, E> {
    /// Gives the concrete verdict its payload back.
    fn into_result(self, verdict: ScopeVerdict) -> Result<R, ScopeStop<E>> {
        match (verdict, self.payload) {
            (Ok(HeldPayload), Some(Ok(value))) => Ok(value),
            (Err(stop), Some(Err(error))) => Err(rejoin_stop(stop, Some(error))),
            (Err(stop), None) => Err(rejoin_stop(stop, None)),
            _ => unreachable!("the concrete scope's verdict names the payload it left held"),
        }
    }
}

/// Parts a domain error from the failure the concrete scope decides on.
fn part_failure<E>(failure: KernelFailure<E>) -> (Option<E>, KernelFailure<HeldPayload>) {
    match failure {
        KernelFailure::Domain(error) => (Some(error), KernelFailure::Domain(HeldPayload)),
        KernelFailure::Stop(stop) => (None, KernelFailure::Stop(stop)),
        KernelFailure::Panic => (None, KernelFailure::Panic),
        KernelFailure::ResultCapacityExceeded => (None, KernelFailure::ResultCapacityExceeded),
    }
}

/// Gives a stop back the domain error held through it.
fn rejoin_stop<E>(stop: ScopeStop<HeldPayload>, error: Option<E>) -> ScopeStop<E> {
    let failure = match (stop, error) {
        (ScopeStop::Admission(denial), None) => return ScopeStop::Admission(denial),
        (ScopeStop::Failure(KernelFailure::Domain(HeldPayload)), Some(error)) => {
            KernelFailure::Domain(error)
        }
        (ScopeStop::Failure(KernelFailure::Stop(stop)), None) => KernelFailure::Stop(stop),
        (ScopeStop::Failure(KernelFailure::Panic), None) => KernelFailure::Panic,
        (ScopeStop::Failure(KernelFailure::ResultCapacityExceeded), None) => {
            KernelFailure::ResultCapacityExceeded
        }
        _ => unreachable!("the concrete scope's verdict names the payload it left held"),
    };
    ScopeStop::Failure(failure)
}

pub(super) fn run_scope_inner<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: Option<u64>,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    hold: Option<ScopeHold<'_, R>>,
    kernel: F,
    payload: PayloadStorage<(), R, E>,
) -> ScopeOutcome<R, E>
where
    F: FnOnce(
        &mut KernelContext<'_, '_>,
        Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>>,
{
    let storage = match payload {
        PayloadStorage::Native => ScopeStorage::Native,
        PayloadStorage::Metered { .. } => ScopeStorage::Metered,
    };
    run_typed_scope(
        lease,
        retained_bytes,
        max_result_bytes,
        charge_parent,
        work_ceiling,
        serial,
        hold,
        &mut Some(kernel),
        payload,
        storage,
    )
}

fn run_typed_scope<R, E>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: Option<u64>,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    hold: Option<ScopeHold<'_, R>>,
    kernel: &mut dyn ScopeKernel<R, E>,
    payload: PayloadStorage<(), R, E>,
    storage: ScopeStorage,
) -> ScopeOutcome<R, E> {
    let (held_reservation, kept) = match hold {
        Some(hold) => (Some(hold.reservation), Some(hold.kept)),
        None => (None, None),
    };
    let mut body = TypedScopeBody {
        kernel,
        payload: None,
        storage: payload,
        kept,
    };
    let (verdict, report) = run_concrete_scope(
        ScopeRequest {
            lease,
            retained_bytes,
            max_result_bytes,
            charge_parent,
            work_ceiling,
            serial,
            hold: held_reservation,
            storage,
            result_slot_bytes: size_of::<R>(),
            error_slot_bytes: size_of::<E>(),
        },
        &mut body,
    );
    ScopeOutcome {
        result: body.into_result(verdict),
        report,
    }
}

/// Only unit payloads may use this metered scope without a heap measurement.
pub(super) fn run_unit_scope(
    lease: Option<&ExecutionResourceLease<'_>>,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    kernel: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<(), KernelFailure<()>>,
) -> ScopeOutcome<(), ()> {
    run_typed_scope(
        lease,
        0,
        Some(0),
        true,
        work_ceiling,
        serial,
        None,
        &mut Some(
            |context: &mut KernelContext<'_, '_>, _: Option<crate::ExecutionResultMemory<'_>>| {
                kernel(context)
            },
        ),
        PayloadStorage::<(), (), ()>::metered(),
        ScopeStorage::Unit,
    )
}
