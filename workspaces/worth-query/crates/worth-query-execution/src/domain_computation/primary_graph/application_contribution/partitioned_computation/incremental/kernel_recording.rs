//! Query retains only receipts issued and funded by the native kernel meter.
use super::super::super::request_execution::QueryRequestExecution;
use super::ExecutionKernelReceipt;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_execution::{ExecutionKernelReceiptStorage, MapKernelContext, MapKernelFailure};
use worth_foundational::PartitionIdentity;

pub(in super::super) struct KernelRecording {
    storage: Option<ExecutionKernelReceiptStorage>,
    receipts: Mutex<BTreeMap<PartitionIdentity, Option<ExecutionKernelReceipt>>>,
}
impl KernelRecording {
    pub(in super::super) fn new(request: &QueryRequestExecution<'_>, retain: bool) -> Self {
        let storage = retain
            .then(|| {
                // The existing admitted request's zero-byte reservation is genuine
                // custody. Execution admits every subsequent receipt allocation.
                let memory = request
                    .reserve(0)
                    .expect("an admitted request reserves zero bytes");
                ExecutionKernelReceiptStorage::new(memory.into_native()).ok()
            })
            .flatten();
        Self {
            storage,
            receipts: Mutex::new(BTreeMap::new()),
        }
    }

    pub(in super::super) fn record<R, E>(
        &self,
        identity: PartitionIdentity,
        context: &mut MapKernelContext<'_, '_>,
        kernel: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>>,
    ) -> Result<R, MapKernelFailure<E>> {
        let Some(storage) = &self.storage else {
            return kernel(context);
        };
        let (result, receipt) = context.record_kernel(storage, kernel);
        self.receipts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(identity, receipt);
        result
    }

    pub(in super::super) fn take(
        &self,
    ) -> BTreeMap<PartitionIdentity, Option<ExecutionKernelReceipt>> {
        std::mem::take(
            &mut *self
                .receipts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}
