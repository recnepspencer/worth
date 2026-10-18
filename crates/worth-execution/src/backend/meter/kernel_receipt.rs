//! Successful checkpoint-only kernels retain cost issued by their native meter.
mod storage;
pub(super) use storage::Capture;
pub use storage::ExecutionKernelReceiptStorage;
use storage::ReceiptMemory;

use super::{KernelContext, KernelFailure, KernelMeter, KernelStop};
use crate::ChargedBytes;
use allocator_api2::vec::Vec;
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Native accepted checkpoints of a successful kernel. This carries cost,
/// never input equality, model currentness or permission to omit a kernel.
#[derive(Clone, Debug)]
pub struct ExecutionKernelReceipt(Arc<Receipt>);

impl ExecutionKernelReceipt {
    /// Cached native cost; reading it grants no input/currentness authority.
    pub fn charged_work(&self) -> u64 {
        self.0.work
    }
}

#[derive(Debug)]
struct Receipt {
    steps: Vec<(u64, u64)>,
    work: u64,
    // Drop admitted steps before their funding.
    _memory: ReceiptMemory,
}

impl PartialEq for ExecutionKernelReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.0.work == other.0.work && self.0.steps == other.0.steps
    }
}
impl Eq for ExecutionKernelReceipt {}

impl ChargedBytes for ExecutionKernelReceipt {
    fn additional_charged_bytes(&self) -> u64 {
        self.0._memory.bytes()
    }
}

struct CaptureScope {
    meter: Rc<RefCell<KernelMeter>>,
    prior: Option<Capture>,
}

impl Drop for CaptureScope {
    fn drop(&mut self) {
        self.meter.borrow_mut().capture = self.prior.take();
    }
}

impl KernelContext<'_, '_> {
    /// Record the actual native meter, not work supplied by the caller.
    /// Storage refusal leaves no receipt and never clears a native refusal.
    /// Context observations and nested stages require actual execution instead.
    pub fn record_kernel<R, E>(
        &mut self,
        storage: &ExecutionKernelReceiptStorage,
        kernel: impl FnOnce(&mut Self) -> Result<R, KernelFailure<E>>,
    ) -> (Result<R, KernelFailure<E>>, Option<ExecutionKernelReceipt>) {
        let before = self.cost();
        let mut prior = self.meter.borrow_mut().capture.take();
        if let Some(prior) = &mut prior {
            prior.invalidate();
        }
        let scope = CaptureScope {
            meter: Rc::clone(&self.meter),
            prior,
        };
        self.meter.borrow_mut().capture = Some(storage.capture());
        let result = kernel(self);
        let (capture, successful) = {
            let mut meter = self.meter.borrow_mut();
            let successful = result.is_ok()
                && meter.checkpoint_stop.is_none()
                && meter.nested_stop.owner().is_none()
                && !meter.pending_request_refusal;
            (meter.capture.take(), successful)
        };
        let cost = self.cost();
        let receipt = capture.and_then(|capture| {
            successful
                .then_some(capture)
                .and_then(|capture| capture.finish(before, cost))
        });
        drop(scope);
        (result, receipt)
    }

    /// Consume genuine prior cost in this native slot. A fitting receipt
    /// takes one real checkpoint; a tight ceiling runs its accepted prefix.
    /// Existing request, nested-stop and first-checkpoint latches remain live.
    pub fn carry_kernel(&mut self, receipt: &ExecutionKernelReceipt) -> Result<(), KernelStop> {
        // A carried stage can descend at a tight ceiling; its fast physical
        // checkpoint must never be recaptured as the original kernel's tape.
        if let Some(capture) = &mut self.meter.borrow_mut().capture {
            capture.invalidate();
        }
        let remaining = {
            let meter = self.meter.borrow();
            meter.limits.ceiling.saturating_sub(meter.work)
        };
        if receipt.0.work <= remaining {
            self.checkpoint(receipt.0.work)
        } else {
            for (units, count) in &receipt.0.steps {
                for _ in 0..*count {
                    self.checkpoint(*units)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
