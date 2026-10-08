use worth_relational::facade::durability::{DurabilityError, RecoveryFailureClass};

use super::super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;
use super::super::{HEADER_BYTES, MINIMUM_V8_ACCEPTED_OUTPUT_BYTES};

/// Proof that the complete frame and every contained wire length fit this host.
pub(super) struct ValidatedCheckpointSize {
    total: usize,
    accepted: usize,
}

impl ValidatedCheckpointSize {
    pub(super) fn new(
        native_len: usize,
        accepted_outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
    ) -> Result<Self, DurabilityError> {
        let accepted = accepted_outputs.iter().try_fold(0_usize, |total, output| {
            let roles = output.roles.iter().try_fold(0_usize, |total, role| {
                checked_sum(&[total, 8, role.role.len(), 1, 8, role.entity_name.len(), 16])
            })?;
            if let Some(resources) = output.resources {
                wire_length(resources.work())?;
                wire_length(resources.retained_bytes())?;
            }
            checked_sum(&[
                total,
                MINIMUM_V8_ACCEPTED_OUTPUT_BYTES - 1,
                output.producer.len(),
                roles,
                output.producer_facts.as_ref().map_or(0, Vec::len),
            ])
        })?;
        let total = checked_sum(&[HEADER_BYTES, native_len, accepted])?;
        wire_length(total)?;
        if total > isize::MAX as usize {
            return Err(size_overflow());
        }
        // Every encoded string, fact payload, role count and output count is
        // bounded by this total, so none can overflow its u64 wire length.
        Ok(Self { total, accepted })
    }

    pub(super) fn total(&self) -> usize {
        self.total
    }
    pub(super) fn accepted(&self) -> usize {
        self.accepted
    }
}

fn checked_sum(lengths: &[usize]) -> Result<usize, DurabilityError> {
    lengths.iter().try_fold(0_usize, |total, length| {
        total.checked_add(*length).ok_or_else(size_overflow)
    })
}

fn wire_length(length: usize) -> Result<u64, DurabilityError> {
    u64::try_from(length).map_err(|_| size_overflow())
}

fn size_overflow() -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::CheckpointSizeOverflow,
        "Query application checkpoint size exceeds this host or wire format",
    )
}
