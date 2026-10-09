use crate::physical_runtime::record_serving::AdoptedExtentCopy;
use worth_store_physical_format::ExtentArenaRange;
use worth_store_wal::WalLsnRange;

/// Bounded observation of actual pre-publication copy effects. This copyable
/// report grants no write, publication, retirement, or retry authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalExtentCopySettlementObservation {
    operation: [u8; 32],
    intent_lsn: u64,
    publication_lsn: u64,
    destination: ExtentArenaRange,
    frame_writes: u32,
    written_bytes: u64,
    receipt_digest: [u8; 32],
}

impl PhysicalExtentCopySettlementObservation {
    pub(in crate::physical_runtime) fn from_capability(
        copy: &AdoptedExtentCopy,
        publication: WalLsnRange,
    ) -> Self {
        Self {
            operation: copy.intent().operation(),
            intent_lsn: copy.durable_intent_lsn(),
            publication_lsn: publication.start().get(),
            destination: copy.intent().destination().arena_range(),
            frame_writes: copy.writes().frames(),
            written_bytes: copy.writes().bytes(),
            receipt_digest: copy.writes().digest(),
        }
    }
    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }
    pub const fn intent_lsn(self) -> u64 {
        self.intent_lsn
    }
    pub const fn publication_lsn(self) -> u64 {
        self.publication_lsn
    }
    pub const fn destination(self) -> ExtentArenaRange {
        self.destination
    }
    /// Includes the one arena-resident manifest write.
    pub const fn frame_writes(self) -> u32 {
        self.frame_writes
    }
    pub const fn written_bytes(self) -> u64 {
        self.written_bytes
    }
    pub const fn receipt_digest(self) -> [u8; 32] {
        self.receipt_digest
    }
}
