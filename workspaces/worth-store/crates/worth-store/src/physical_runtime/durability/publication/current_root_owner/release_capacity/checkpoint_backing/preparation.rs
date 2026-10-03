//! Prepare the next post-drop capture, including the largest prefix fold.
use super::super::{
    backing::{
        vector_heap_bytes, LiveBackingWindow, ReleasePublicationAllocationOwner,
        SHARED_CUSTODY_BYTES,
    },
    heads::SelectedReleaseHeadRoster,
    ReleaseCertificateCapacityDenial as Denial, SelectedReleaseCustodyLedger,
};
use super::storage::*;
use crate::physical_runtime::{
    durability::FundedCheckpointCommandBuffer, PhysicalRecoveryAllocationAdmission,
};
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES, MAX_CHECKPOINT_BINDING_RECORD_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_BYTES, MAX_CHECKPOINT_CERTIFICATE_RECORDS,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
    RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};

const COMMAND_BYTES: usize =
    MAX_CHECKPOINT_BINDING_RECORD_BYTES + CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES;
const fn arc_bytes<T>() -> u64 {
    (std::mem::size_of::<T>() + 2 * std::mem::size_of::<usize>()) as u64
}

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn checkpoint_backing_requirement(
        &self,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
        tier: bool,
    ) -> Result<u64, Denial> {
        let batches = self
            .pending_drop_count()
            .checked_add(usize::from(incoming.is_some()))
            .ok_or(Denial::CapacityExhausted)?;
        let records = batches + 1 + usize::from(tier);
        let payload = RELEASE_CHECKPOINT_BATCH_WIRE_BYTES
            .max(RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES)
            .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
            .max(if tier {
                TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES
            } else {
                0
            });
        let inserts = self
            .pending_events
            .iter()
            .filter(|event| {
                matches!(
                    event.head_step().mutation(),
                    ReleaseCustodyHeadMutationV1::Upsert {
                        expected_prior: None,
                        ..
                    }
                )
            })
            .count();
        let heads = usize::try_from(self.checkpoint_heads.len())
            .ok()
            .and_then(|count| count.checked_add(inserts))
            .and_then(|count| {
                count.checked_add(usize::from(
                    incoming.is_some_and(|key| self.effective_heads.head(key).is_none()),
                ))
            })
            .ok_or(Denial::CapacityExhausted)?;
        let bytes = batches
            .checked_mul(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES + 20)
            .and_then(|bytes| {
                bytes.checked_add(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES
                        .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
                        + 20,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(if tier {
                    TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES + 20
                } else {
                    0
                })
            })
            .and_then(|bytes| {
                bytes.checked_add(records.checked_mul(std::mem::size_of::<CertificateRange>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    payload.max(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes()),
                )
            })
            .and_then(|bytes| bytes.checked_add(payload + 20 + COMMAND_BYTES))
            .and_then(|bytes| {
                bytes.checked_add(heads.checked_mul(std::mem::size_of::<
                    worth_store_physical_format::ReleaseCustodyHeadEntryV1,
                >())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(batches.checked_mul(std::mem::size_of::<
                    worth_store_physical_format::ReleaseCheckpointBatchV1,
                >())?)
            })
            .and_then(|bytes| bytes.checked_add(payload))
            .ok_or(Denial::CapacityExhausted)?;
        (bytes as u64)
            .checked_add(
                2 * SHARED_CUSTODY_BYTES
                    + arc_bytes::<SealedCheckpointStorage>()
                    + arc_bytes::<ReusableCheckpointSlot>()
                    + arc_bytes::<FundedCheckpointBufferPreparation>(),
            )
            .ok_or(Denial::CapacityExhausted)
    }
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn prepare_checkpoint_backing(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
        tier: bool,
    ) -> Result<(), Denial> {
        let batches = self
            .pending_drop_count()
            .checked_add(usize::from(incoming.is_some()))
            .ok_or(Denial::CapacityExhausted)?;
        let records = batches + 1 + usize::from(tier);
        let payload_max = RELEASE_CHECKPOINT_BATCH_WIRE_BYTES
            .max(RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES)
            .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
            .max(if tier {
                TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES
            } else {
                0
            });
        let framed = batches
            .checked_mul(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES + 20)
            .and_then(|bytes| {
                bytes.checked_add(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES
                        .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
                        + 20,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(if tier {
                    TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES + 20
                } else {
                    0
                })
            })
            .ok_or(Denial::CapacityExhausted)?;
        if records as u64 > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || framed as u64 > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(Denial::CapacityExhausted);
        }
        let inserted = self
            .pending_events
            .iter()
            .filter(|event| {
                matches!(
                    event.head_step().mutation(),
                    ReleaseCustodyHeadMutationV1::Upsert {
                        expected_prior: None,
                        ..
                    }
                )
            })
            .count();
        let heads = usize::try_from(self.checkpoint_heads.len())
            .ok()
            .and_then(|count| count.checked_add(inserted))
            .and_then(|count| {
                count.checked_add(usize::from(
                    incoming.is_some_and(|key| self.effective_heads.head(key).is_none()),
                ))
            })
            .ok_or(Denial::CapacityExhausted)?;
        if self.checkpoint_backing.is_none() {
            // Slot allocation is retained with the observer grant prepared below.
            let mut preparation = empty();
            preparation.grow(owner, ceiling, batches, records, framed, payload_max, heads)?;
            self.checkpoint_backing = Some(Arc::new(ReusableCheckpointSlot {
                available: Mutex::new(Some(Arc::new(SealedCheckpointStorage::from_prepared(
                    preparation,
                )))),
            }));
            return Ok(());
        }
        let slot = self.checkpoint_backing.as_ref().unwrap();
        let mut available = slot.available.lock().unwrap_or_else(|p| p.into_inner());
        if available.as_ref().is_none_or(|capsule| {
            Arc::strong_count(capsule) != 1 || !capsule.returned_command_is_exclusive()
        }) {
            let mut preparation = empty();
            preparation.grow(owner, ceiling, batches, records, framed, payload_max, heads)?;
            *available = Some(Arc::new(SealedCheckpointStorage::from_prepared(
                preparation,
            )));
            return Ok(());
        }
        let capsule = Arc::get_mut(available.as_mut().unwrap()).expect("sole slot owner");
        let mut preparation = capsule.take_preparation();
        let result = preparation.grow(owner, ceiling, batches, records, framed, payload_max, heads);
        capsule.put_preparation(preparation);
        result
    }
}

fn empty() -> CheckpointPreparation {
    CheckpointPreparation {
        observer: SnapshotBytes {
            bytes: Vec::new(),
            records: Vec::new(),
            scratch: Vec::new(),
            frame_scratch: Vec::new(),
            custody: None,
        },
        fold: None,
        command: None,
    }
}

impl CheckpointPreparation {
    pub(super) fn grow(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        batches: usize,
        records: usize,
        framed: usize,
        payload_max: usize,
        heads: usize,
    ) -> Result<(), Denial> {
        let retained = vector_heap_bytes(&self.observer.bytes)
            .and_then(|bytes| bytes.checked_add(vector_heap_bytes(&self.observer.records)?))
            .and_then(|bytes| bytes.checked_add(vector_heap_bytes(&self.observer.scratch)?))
            .and_then(|bytes| bytes.checked_add(vector_heap_bytes(&self.observer.frame_scratch)?))
            .ok_or(Denial::CapacityExhausted)?;
        let controls = SHARED_CUSTODY_BYTES
            + arc_bytes::<SealedCheckpointStorage>()
            + arc_bytes::<ReusableCheckpointSlot>()
            + arc_bytes::<FundedCheckpointBufferPreparation>();
        let already_live = retained
            .checked_add(controls)
            .and_then(|bytes| {
                bytes.checked_add(if self.command.is_some() {
                    COMMAND_BYTES as u64
                } else {
                    0
                })
            })
            .ok_or(Denial::CapacityExhausted)?;
        let mut window =
            LiveBackingWindow::new(owner, &mut self.observer.custody, ceiling, already_live)?;
        grow_to(&mut window, &mut self.observer.bytes, framed)?;
        grow_to(&mut window, &mut self.observer.records, records)?;
        grow_to(
            &mut window,
            &mut self.observer.scratch,
            payload_max.max(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes()),
        )?;
        grow_to(
            &mut window,
            &mut self.observer.frame_scratch,
            payload_max + 20,
        )?;
        if self.command.is_none() {
            let bytes = window.reserve_vec(COMMAND_BYTES)?;
            drop(window);
            self.command = Some(FundedCheckpointCommandBuffer::from_prepared(
                FundedCheckpointBufferPreparation {
                    bytes,
                    _custody: Arc::clone(self.observer.custody.as_ref().unwrap()),
                },
            ));
        } else {
            drop(window);
        }
        if self.fold.is_none() {
            self.fold = Some(FoldWorkspace {
                heads: SelectedReleaseHeadRoster::empty(),
                batches: Vec::new(),
                scratch: Vec::new(),
                custody: None,
            });
        }
        let fold = self.fold.as_mut().unwrap();
        let retained = fold
            .heads
            .owned_heap_bytes()
            .and_then(|bytes| bytes.checked_add(vector_heap_bytes(&fold.batches)?))
            .and_then(|bytes| bytes.checked_add(vector_heap_bytes(&fold.scratch)?))
            .ok_or(Denial::CapacityExhausted)?;
        let mut window = LiveBackingWindow::new(
            owner,
            &mut fold.custody,
            ceiling,
            retained + SHARED_CUSTODY_BYTES,
        )?;
        fold.heads.prepare_fold_capacity(heads, &mut window)?;
        grow_to(&mut window, &mut fold.batches, batches)?;
        grow_to(&mut window, &mut fold.scratch, payload_max)?;
        Ok(())
    }
}
fn grow_to<T>(
    window: &mut LiveBackingWindow<'_>,
    values: &mut Vec<T>,
    count: usize,
) -> Result<(), Denial> {
    let additional = count.saturating_sub(values.len());
    window.grow_vec(values, additional)
}
