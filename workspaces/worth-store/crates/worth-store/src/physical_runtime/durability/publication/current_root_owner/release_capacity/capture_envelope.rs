//! One checkpoint-capture envelope: every byte the capture's certificate
//! custody, independent fold and in-flight commands can hold. Capture admits
//! it once, before any effect, against the Store's Recovery budget.
mod reservation;
mod storage;
#[cfg(test)]
mod tests;
pub(in crate::physical_runtime) use storage::{
    CheckpointCertificateFrame, CheckpointCertificateViews,
};
pub(in crate::physical_runtime::durability::publication::current_root_owner) use storage::{
    CheckpointPreparation, SealedCheckpointStorage,
};

use std::sync::Arc;

use worth_store_physical_format::{
    DurablePhysicalRootManifest, ReleaseCheckpointBatchV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES, MAX_CHECKPOINT_BINDING_RECORD_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_BYTES, MAX_CHECKPOINT_CERTIFICATE_RECORDS,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES, RELEASE_CHECKPOINT_BATCH_WIRE_BYTES,
    RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};

use super::super::certificate_capacity::CheckpointCustodyDenial;
use super::backing::{LiveReleaseAllocation, SHARED_CUSTODY_BYTES};
use super::{ReleaseCertificateCapacityDenial as Denial, SelectedReleaseCustodyLedger};
use storage::{CertificateRange, FoldWorkspace, SnapshotBytes};

const FRAME: usize = CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES;
/// The largest checkpoint command is one framed binding record. The executor
/// may still hold the previous command while the next one is encoded.
const COMMAND_BYTES: usize = MAX_CHECKPOINT_BINDING_RECORD_BYTES + FRAME;
const IN_FLIGHT_COMMANDS: usize = 2;
const SEALED_STORAGE_BYTES: usize =
    std::mem::size_of::<SealedCheckpointStorage>() + 2 * std::mem::size_of::<usize>();

/// The exact capacities one capture may use, bounded by the certificate
/// limits, the release-head population and the command frame. The standing
/// reservation holds the envelope that includes each admitted drop's Batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct CheckpointCaptureEnvelope
{
    batches: usize,
    records: usize,
    framed: usize,
    payload: usize,
    heads: usize,
    bytes: u64,
}

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn checkpoint_capture_envelope(
        &self,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
        tier: bool,
        scan: u64,
    ) -> Result<CheckpointCaptureEnvelope, Denial> {
        let envelope = self.capture_envelope_bytes(incoming, tier, scan)?;
        if envelope.records as u64 > MAX_CHECKPOINT_CERTIFICATE_RECORDS
            || envelope.framed as u64 > MAX_CHECKPOINT_CERTIFICATE_BYTES
        {
            return Err(Denial::CapacityExhausted);
        }
        Ok(envelope)
    }

    /// The envelope's capacities without the certificate limits, so the
    /// standing reservation can also hold a not-yet-activated tier slot.
    /// `scan` is the checkpoint pin scan's admitted bound.
    fn capture_envelope_bytes(
        &self,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
        tier: bool,
        scan: u64,
    ) -> Result<CheckpointCaptureEnvelope, Denial> {
        let batches = self.pending_drop_count() + usize::from(incoming.is_some());
        let records = batches + 1 + usize::from(tier);
        let tier_payload = if tier {
            TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES
        } else {
            0
        };
        let framed = batches
            .checked_mul(RELEASE_CHECKPOINT_BATCH_WIRE_BYTES + FRAME)
            .and_then(|bytes| {
                bytes.checked_add(
                    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES
                        .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
                        + FRAME,
                )
            })
            .and_then(|bytes| bytes.checked_add(if tier { tier_payload + FRAME } else { 0 }))
            .ok_or(Denial::CapacityExhausted)?;
        let payload = RELEASE_CHECKPOINT_BATCH_WIRE_BYTES
            .max(RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES)
            .max(RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES)
            .max(tier_payload);
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
        // Scratch, frame scratch, fold scratch, commands and the shared storage.
        let fixed = payload.max(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes())
            + payload
            + FRAME
            + payload
            + IN_FLIGHT_COMMANDS * COMMAND_BYTES
            + SEALED_STORAGE_BYTES;
        let bytes = heads
            .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>())
            .and_then(|bytes| bytes.checked_add(framed))
            .and_then(|bytes| bytes.checked_add(fixed))
            .and_then(|bytes| {
                bytes.checked_add(records.checked_mul(std::mem::size_of::<CertificateRange>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    batches.checked_mul(std::mem::size_of::<ReleaseCheckpointBatchV1>())?,
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(SHARED_CUSTODY_BYTES))
            .and_then(|bytes| bytes.checked_add(scan))
            .ok_or(Denial::CapacityExhausted)?;
        Ok(CheckpointCaptureEnvelope {
            batches,
            records,
            framed,
            payload,
            heads,
            bytes,
        })
    }
}

impl CheckpointCaptureEnvelope {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) const fn bytes(
        self,
    ) -> u64 {
        self.bytes
    }

    /// Plain allocations at their admitted capacities; the custody keeps the
    /// admitted grant alive exactly as long as these buffers.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn prepare(
        self,
        custody: Arc<LiveReleaseAllocation>,
    ) -> CheckpointPreparation {
        CheckpointPreparation {
            observer: SnapshotBytes {
                bytes: Vec::with_capacity(self.framed),
                records: Vec::with_capacity(self.records),
                scratch: Vec::with_capacity(
                    self.payload
                        .max(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes()),
                ),
                frame_scratch: Vec::with_capacity(self.payload + FRAME),
            },
            fold: FoldWorkspace {
                heads: super::heads::SelectedReleaseHeadRoster::with_fold_capacity(self.heads),
                batches: Vec::with_capacity(self.batches),
                scratch: Vec::with_capacity(self.payload),
            },
            custody,
        }
    }
}

impl From<Denial> for CheckpointCustodyDenial {
    fn from(value: Denial) -> Self {
        match value {
            Denial::Resident(cause) => Self::Backing(cause),
            _ => Self::ReleaseCertificateUnavailable,
        }
    }
}
