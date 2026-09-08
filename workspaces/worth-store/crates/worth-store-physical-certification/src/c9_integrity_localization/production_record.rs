//! Known submitted payloads paired with identities from actual completed writes.
use super::production_profile::ProductionWorldProfile;
use serde::{Deserialize, Serialize};
use worth_store::physical_runtime::{CompletedPhysicalMutation, ExternalPhysicalRecordLocator};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ProducedRecord {
    epoch: [u8; 16],
    ordinal: u64,
    pub(super) length: usize,
    pub(super) byte: u8,
}

impl ProducedRecord {
    pub(super) fn completed(
        completed: &CompletedPhysicalMutation,
        batch: usize,
        profile: ProductionWorldProfile,
    ) -> Vec<Self> {
        let inline = profile.inline_records_per_batch(batch);
        assert_eq!(
            completed.persisted_records().len(),
            inline + usize::from(profile == ProductionWorldProfile::Primary16KiB)
        );
        completed
            .persisted_records()
            .iter()
            .enumerate()
            .map(|(index, record)| Self {
                epoch: record.allocation_epoch(),
                ordinal: record.ordinal(),
                length: if index < inline {
                    profile.inline_record_bytes()
                } else {
                    64 * 1024
                },
                byte: if index < inline {
                    (batch * 17 + index) as u8
                } else {
                    0xC9 ^ batch as u8
                },
            })
            .collect()
    }

    pub(super) fn locator(&self, store: [u8; 16]) -> ExternalPhysicalRecordLocator {
        let mut encoded = [0; 40];
        encoded[..16].copy_from_slice(&store);
        encoded[16..32].copy_from_slice(&self.epoch);
        encoded[32..].copy_from_slice(&self.ordinal.to_le_bytes());
        ExternalPhysicalRecordLocator::decode(encoded).unwrap()
    }
}
