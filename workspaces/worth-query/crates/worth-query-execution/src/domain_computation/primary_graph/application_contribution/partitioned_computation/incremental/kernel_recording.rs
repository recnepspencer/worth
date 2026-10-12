//! Optional checkpoint retention owns and reserves its in-flight allocations.
use super::super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::KernelSteps;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_foundational::PartitionIdentity;

pub(in super::super) struct KernelRecording {
    limit: u64,
    memory: Mutex<Option<QueryMemoryReservation>>,
    receipts: Mutex<BTreeMap<PartitionIdentity, Option<KernelSteps>>>,
}
impl KernelRecording {
    pub(in super::super) fn new(
        request: &QueryRequestExecution<'_>,
        retain: bool,
        limit: u64,
    ) -> Self {
        Self {
            limit,
            // Preparation already admitted this request's lease. room::check
            // tests requested > room, so a zero-byte hold cannot be refused,
            // even with a full lineage or a cancelled request.
            memory: Mutex::new(retain.then(|| {
                request
                    .reserve(0)
                    .expect("an admitted request reserves zero bytes")
            })),
            receipts: Mutex::new(BTreeMap::new()),
        }
    }
    pub(in super::super) fn start(&self) -> Option<KernelSteps> {
        self.memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|_| KernelSteps::default())
    }
    pub(in super::super) fn accepted(&self, steps: &mut Option<KernelSteps>, work: u64) {
        let Some(recorded) = steps else {
            return;
        };
        // At most 2 * accepted work + 1 RLE entries; all vector capacities
        // fit each partition's declared retained bytes; their aggregate is
        // reserved against the request before any vector grows.
        let recorded_bytes = worth_execution::ChargedBytes::additional_charged_bytes(recorded);
        if !recorded.accepted(work, |bytes| {
            let mut memory = self
                .memory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            memory.as_mut().is_some_and(|held| {
                recorded_bytes
                    .checked_add(bytes)
                    .is_some_and(|n| n <= self.limit)
                    && held.grow(bytes).is_ok()
            })
        }) {
            *steps = None;
        }
    }
    pub(in super::super) fn finish(
        &self,
        identity: PartitionIdentity,
        steps: Option<KernelSteps>,
        cost: (u64, u64),
    ) {
        if self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            return;
        }
        // A nested public pattern can add cost not represented by owner advances.
        // Such a kernel cannot be replayed and leaves the next run unretained.
        let steps = steps.filter(|s| {
            let units = s.units();
            cost == (units, units)
        });
        self.receipts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(identity, steps);
    }
    pub(in super::super) fn take(
        &self,
    ) -> (
        BTreeMap<PartitionIdentity, Option<KernelSteps>>,
        Option<QueryMemoryReservation>,
    ) {
        (
            std::mem::take(
                &mut *self
                    .receipts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            ),
            self.memory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
        )
    }
}
