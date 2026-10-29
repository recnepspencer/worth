//! Typed scope payload. The concrete driver sees it only through [`ScopeSlot`].
use crate::authority::LeaseDenial;
use crate::backend::meter::{
    kernel_failure_class, KernelContext, KernelFailure, KernelStop, NestedStopOwner,
};

use super::ScopeStop;

/// The shell's typed result, driven by the one concrete scope lifecycle.
pub(super) trait ScopeSlot {
    fn store_stop(&mut self, stop: KernelStop);
    fn drop_produced(&mut self);
    fn produced_is_ok(&self) -> bool;
    fn enforce_result_cap(&mut self, cap: u64);
    fn measured_from_produced(&self) -> u64;
    fn map_produced_to_stop(&mut self);
    fn store_admission(&mut self, denial: LeaseDenial);
    fn measured_kept_bytes(&self) -> u64;
    fn kept_for_hold(&self) -> Result<u64, LeaseDenial>;
    fn settled_succeeded(&self) -> bool;
    fn refuse_success(&mut self, denial: LeaseDenial);
    fn stop_owner(&mut self, context: &KernelContext<'_, '_>) -> Option<NestedStopOwner>;
}

pub(super) struct TypedSlot<'a, R, E> {
    produced: Option<Result<R, KernelFailure<E>>>,
    settled: Option<Result<R, ScopeStop<E>>>,
    result_measure: Option<fn(&R) -> u64>,
    error_measure: Option<fn(&E) -> u64>,
    kept: Option<&'a dyn Fn(&R) -> Option<u64>>,
}

impl<'a, R, E> TypedSlot<'a, R, E> {
    pub(super) fn new(
        result_measure: Option<fn(&R) -> u64>,
        error_measure: Option<fn(&E) -> u64>,
        kept: Option<&'a dyn Fn(&R) -> Option<u64>>,
    ) -> Self {
        Self {
            produced: None,
            settled: None,
            result_measure,
            error_measure,
            kept,
        }
    }

    pub(super) fn store_produced(&mut self, result: Result<R, KernelFailure<E>>) {
        self.produced = Some(result);
    }

    pub(super) fn into_result(self) -> Result<R, ScopeStop<E>> {
        self.settled.expect("scope drive settled a result")
    }
}

impl<R, E> ScopeSlot for TypedSlot<'_, R, E> {
    fn store_stop(&mut self, stop: KernelStop) {
        self.produced = Some(Err(KernelFailure::Stop(stop)));
    }

    fn drop_produced(&mut self) {
        drop(self.produced.take());
    }

    fn produced_is_ok(&self) -> bool {
        matches!(self.produced, Some(Ok(_)))
    }

    fn enforce_result_cap(&mut self, cap: u64) {
        let exceeds = match &self.produced {
            Some(Ok(value)) => self
                .result_measure
                .is_some_and(|measure| measure(value) > cap),
            Some(Err(KernelFailure::Domain(error))) => self
                .error_measure
                .is_some_and(|measure| measure(error) > cap),
            _ => false,
        };
        if exceeds {
            self.produced = Some(Err(KernelFailure::ResultCapacityExceeded));
        }
    }

    fn measured_from_produced(&self) -> u64 {
        match &self.produced {
            Some(Ok(value)) => self.result_measure.expect("measured result callback")(value),
            Some(Err(KernelFailure::Domain(error))) => {
                self.error_measure.expect("measured error callback")(error)
            }
            _ => 0,
        }
    }

    fn map_produced_to_stop(&mut self) {
        let produced = self
            .produced
            .take()
            .expect("scope produced a kernel result");
        self.settled = Some(produced.map_err(ScopeStop::Failure));
    }

    fn store_admission(&mut self, denial: LeaseDenial) {
        self.settled = Some(Err(ScopeStop::Admission(denial)));
    }

    fn measured_kept_bytes(&self) -> u64 {
        match &self.settled {
            Some(Ok(value)) => self.result_measure.expect("measured result callback")(value),
            Some(Err(ScopeStop::Failure(KernelFailure::Domain(error)))) => {
                self.error_measure.expect("measured error callback")(error)
            }
            _ => 0,
        }
    }

    fn kept_for_hold(&self) -> Result<u64, LeaseDenial> {
        match &self.settled {
            Some(Ok(value)) => self.kept.expect("held scope carries its kept charge")(value)
                .ok_or_else(LeaseDenial::charged_bytes_overflow),
            Some(Err(_)) => Ok(0),
            None => unreachable!("settlement follows a scope result"),
        }
    }

    fn settled_succeeded(&self) -> bool {
        matches!(self.settled, Some(Ok(_)))
    }

    fn refuse_success(&mut self, denial: LeaseDenial) {
        self.settled = Some(Err(ScopeStop::Admission(denial)));
    }

    fn stop_owner(&mut self, context: &KernelContext<'_, '_>) -> Option<NestedStopOwner> {
        let failed = self
            .settled
            .as_ref()
            .expect("scope settlement classifies a result")
            .is_err();
        context.note_settled_failure(failed);
        match self
            .settled
            .as_ref()
            .expect("scope settlement classifies a result")
        {
            Ok(_) => None,
            Err(ScopeStop::Admission(_)) => Some(NestedStopOwner::Request),
            Err(ScopeStop::Failure(failure)) => {
                Some(context.owner_of_failure_class(kernel_failure_class(failure)))
            }
        }
    }
}
