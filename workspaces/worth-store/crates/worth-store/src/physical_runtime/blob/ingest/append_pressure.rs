use super::super::{BlobIngestAllocation, BlobMemoryDenial, BlobResidentComponent};

const TRANSIENT_COPIES: [(BlobResidentComponent, u64); 5] = [
    (BlobResidentComponent::EncodedChunk, 1),
    (BlobResidentComponent::CanonicalRedo, 3),
    (BlobResidentComponent::Writeback, 3),
    (BlobResidentComponent::WalFrame, 1),
    (BlobResidentComponent::Scratch, 1),
];

/// Conservatively reserves simultaneous C5 payload materializations for one
/// synchronous typed blob append. The source-window and tree-frontier charges
/// remain held by the ingest owner while this guard is live.
pub(super) struct BlobAppendPressure<'charge, 'runtime> {
    allocation: &'charge mut BlobIngestAllocation<'runtime>,
}

impl<'charge, 'runtime> BlobAppendPressure<'charge, 'runtime> {
    pub(super) fn admit(
        allocation: &'charge mut BlobIngestAllocation<'runtime>,
        encoded_bytes: u64,
    ) -> Result<Self, BlobMemoryDenial> {
        for (component, copies) in TRANSIENT_COPIES {
            let bytes =
                encoded_bytes
                    .checked_mul(copies)
                    .ok_or(BlobMemoryDenial::CeilingExceeded {
                        requested: u64::MAX,
                        maximum: allocation.observation().ceiling(),
                    })?;
            allocation.set_live(component, bytes)?;
        }
        Ok(Self { allocation })
    }
}

impl Drop for BlobAppendPressure<'_, '_> {
    fn drop(&mut self) {
        for (component, _) in TRANSIENT_COPIES {
            self.allocation
                .set_live(component, 0)
                .expect("releasing an admitted append charge cannot fail");
        }
    }
}
