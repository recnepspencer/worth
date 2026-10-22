//! Current-dispatch captures filled only after native evaluation precedence.
use std::sync::OnceLock;

use super::super::{
    admission::BatchDeclaration,
    port::TaskOutcome,
    retained_result::{result_bytes, result_layout_bytes, ResultStorageDenial},
    task_result::TaskResult,
    ExecutionKernelReceipt, ExecutionKernelReceiptStorage,
};
use crate::{
    ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder, ExecutionArrayIntoIter,
    ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial,
};

pub(crate) struct EvaluatedTask {
    pub(crate) accounted: bool,
    pub(crate) work: u64,
    pub(crate) span: u64,
    pub(crate) receipt: Option<ExecutionKernelReceipt>,
    pub(crate) result_heap: Option<u64>,
}

pub(crate) struct SettlementSlot {
    pub(crate) evaluated: OnceLock<EvaluatedTask>,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}

/// Temporary slots own the future result grants across the native run's
/// admission release. Each accepted result takes only its own grant.
pub(crate) struct SettlementRecords {
    slots: ExecutionArray<SettlementSlot>,
    storage: Option<ExecutionKernelReceiptStorage>,
    _scratch: Option<ExecutionMemoryReservation>,
}

enum PayloadStorage {
    Declared,
    Native,
}

impl SettlementRecords {
    pub(crate) fn admit<R>(
        batch: &BatchDeclaration,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<Self, ResultStorageDenial> {
        Self::admit_with::<R>(batch, lease, |_| true)
    }

    pub(crate) fn admit_with<R>(
        batch: &BatchDeclaration,
        lease: Option<&ExecutionResourceLease<'_>>,
        new_result: impl FnMut(usize) -> bool,
    ) -> Result<Self, ResultStorageDenial> {
        Self::admit_storage::<R>(batch, lease, new_result, PayloadStorage::Declared)
    }

    pub(crate) fn admit_native<R>(
        batch: &BatchDeclaration,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<Self, ResultStorageDenial> {
        Self::admit_storage::<R>(batch, lease, |_| true, PayloadStorage::Native)
    }

    fn admit_storage<R>(
        batch: &BatchDeclaration,
        lease: Option<&ExecutionResourceLease<'_>>,
        mut new_result: impl FnMut(usize) -> bool,
        payload_storage: PayloadStorage,
    ) -> Result<Self, ResultStorageDenial> {
        // Current captures and the returned handle vector coexist while
        // native values move into their separately funded cells.
        let accounted = !matches!(payload_storage, PayloadStorage::Native)
            || super::super::meter::RunLimits::native_accounting(lease);
        let bytes = if accounted {
            batch
                .len()
                .checked_mul(
                    size_of::<SettlementSlot>()
                        + size_of::<super::super::ExecutionRetainedResult<R>>(),
                )
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(LeaseDenial::charged_bytes_overflow)?
        } else {
            0
        };
        let scratch = if accounted {
            Some(ExecutionMemoryReservation::reserve_in_scope(lease, bytes)?)
        } else {
            None
        };
        let storage = if accounted {
            Some(
                ExecutionKernelReceiptStorage::new(ExecutionMemoryReservation::reserve_in_scope(
                    lease, 0,
                )?)
                .map_err(LeaseDenial::MemoryExhausted)?,
            )
        } else {
            None
        };
        // The explicit scope grant above also covers serial backing.
        let mut slots = ExecutionArrayBuilder::allocate(
            batch.len(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(ResultStorageDenial::Allocation)?;
        for index in 0..batch.len() {
            let bytes = if accounted {
                Some(
                    match payload_storage {
                        PayloadStorage::Declared => {
                            let capacity = batch.result_capacity(index).ok_or(
                                ResultStorageDenial::RequiresExecution(batch.identities()[index]),
                            )?;
                            result_bytes::<R>(capacity)
                        }
                        PayloadStorage::Native => result_layout_bytes::<R>(),
                    }
                    .ok_or_else(LeaseDenial::charged_bytes_overflow)?,
                )
            } else {
                None
            };
            slots
                .push(SettlementSlot {
                    evaluated: OnceLock::new(),
                    memory: if let Some(bytes) = bytes.filter(|_| new_result(index)) {
                        Some(ExecutionMemoryReservation::reserve_in_scope(lease, bytes)?)
                    } else {
                        None
                    },
                })
                .map_err(ResultStorageDenial::Allocation)?;
        }
        Ok(Self {
            slots: slots.seal().map_err(ResultStorageDenial::Allocation)?,
            storage,
            _scratch: scratch,
        })
    }

    pub(in crate::backend) fn storage(&self) -> Option<&ExecutionKernelReceiptStorage> {
        self.storage.as_ref()
    }

    pub(in crate::backend) fn record<R, E>(
        &self,
        index: usize,
        outcome: &TaskOutcome<R, E>,
        receipt: Option<ExecutionKernelReceipt>,
        result_heap: Option<u64>,
        accounted: bool,
    ) {
        let receipt = if matches!(&outcome.result, TaskResult::Value(_)) {
            receipt
        } else {
            None
        };
        self.slots[index]
            .evaluated
            .set(EvaluatedTask {
                accounted,
                work: outcome.work,
                span: outcome.span,
                receipt,
                result_heap,
            })
            .unwrap_or_else(|_| panic!("one native evaluation per settlement slot"));
    }

    pub(crate) fn into_slots(self) -> impl Iterator<Item = SettlementSlot> {
        IntoSlots {
            slots: self.slots.into_iter(),
            _storage: self.storage,
            _scratch: self._scratch,
        }
    }
}

struct IntoSlots {
    slots: ExecutionArrayIntoIter<SettlementSlot>,
    _storage: Option<ExecutionKernelReceiptStorage>,
    _scratch: Option<ExecutionMemoryReservation>,
}
impl Iterator for IntoSlots {
    type Item = SettlementSlot;
    fn next(&mut self) -> Option<Self::Item> {
        self.slots.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.slots.size_hint()
    }
}
