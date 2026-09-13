use sha2::{Digest, Sha256};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecoveryStaticConfiguration {
    identity: [u8; 32],
    record_format: PhysicalRecordFormatDeclaration,
}

impl PhysicalRecoveryStaticConfiguration {
    pub fn current() -> Self {
        let record_format = PhysicalRecordFormatDeclaration::builder()
            .admit()
            .expect("the canonical physical record format is supported");
        Self::for_record_format(record_format)
    }

    pub fn for_record_format(record_format: PhysicalRecordFormatDeclaration) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth.store.physical.recovery.configuration@1");
        digest.update(record_format.canonical_identity_bytes());
        Self {
            identity: digest.finalize().into(),
            record_format,
        }
    }

    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub(crate) const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }
}
