use worth_store_security::{
    StoreCurrentSecurityScopeWitnessSet, StoreKeyVersionPosture, StoreLegacySecurityPosture,
    StoreSecurityMetadata, StoreSecurityScopeIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreWalRecordIdentity {
    sequence: u64,
}

impl StoreWalRecordIdentity {
    pub const fn new(sequence: u64) -> Self {
        Self { sequence }
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreCheckpointRecordIdentity {
    checkpoint_epoch: u64,
}

impl StoreCheckpointRecordIdentity {
    pub const fn new(checkpoint_epoch: u64) -> Self {
        Self { checkpoint_epoch }
    }

    pub const fn checkpoint_epoch(self) -> u64 {
        self.checkpoint_epoch
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalSecurityMetadataCarrier {
    physical_metadata: StoreSecurityMetadata,
    security_identity: StoreSecurityScopeIdentity,
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

impl WalSecurityMetadataCarrier {
    pub fn for_wal_record(
        witnesses: &StoreCurrentSecurityScopeWitnessSet,
        key_version_posture: StoreKeyVersionPosture,
        legacy_posture: StoreLegacySecurityPosture,
    ) -> Self {
        Self {
            physical_metadata: StoreSecurityMetadata::from_current_security_scope(
                witnesses,
                key_version_posture,
                legacy_posture,
            ),
            security_identity: witnesses.key_scope().identity(),
            authority_identity: witnesses.authority_identity(),
        }
    }

    pub fn for_checkpoint_record(
        witnesses: &StoreCurrentSecurityScopeWitnessSet,
        key_version_posture: StoreKeyVersionPosture,
        legacy_posture: StoreLegacySecurityPosture,
    ) -> Self {
        Self::for_wal_record(witnesses, key_version_posture, legacy_posture)
    }

    pub const fn physical_metadata(self) -> StoreSecurityMetadata {
        self.physical_metadata
    }

    pub const fn key_scope(self) -> worth_store_security::StoreKeyScope {
        self.physical_metadata.key_scope()
    }

    pub const fn tenant_scope(self) -> worth_store_security::StoreTenantScope {
        self.physical_metadata.tenant_scope()
    }

    pub const fn security_identity(self) -> StoreSecurityScopeIdentity {
        self.security_identity
    }

    pub const fn authority_identity(self) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.authority_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalSecurityMetadataEnvelope<T> {
    record: T,
    security_metadata: WalSecurityMetadataCarrier,
}

impl<T> WalSecurityMetadataEnvelope<T> {
    pub const fn wal_record(record: T, security_metadata: WalSecurityMetadataCarrier) -> Self {
        Self {
            record,
            security_metadata,
        }
    }

    pub const fn checkpoint_record(
        record: T,
        security_metadata: WalSecurityMetadataCarrier,
    ) -> Self {
        Self {
            record,
            security_metadata,
        }
    }

    pub const fn record(&self) -> &T {
        &self.record
    }

    pub const fn security_metadata(&self) -> WalSecurityMetadataCarrier {
        self.security_metadata
    }
}

pub type WalRecordSecurityMetadataEnvelope = WalSecurityMetadataEnvelope<StoreWalRecordIdentity>;
pub type CheckpointRecordSecurityMetadataEnvelope =
    WalSecurityMetadataEnvelope<StoreCheckpointRecordIdentity>;

impl worth_store_security::RecoveryWalRecordSecurityMetadataSource
    for WalSecurityMetadataEnvelope<StoreWalRecordIdentity>
{
    fn recovery_sequence(&self) -> u64 {
        self.record().sequence()
    }

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata {
        self.security_metadata().physical_metadata()
    }
}

impl worth_store_security::RecoveryCheckpointRecordSecurityMetadataSource
    for WalSecurityMetadataEnvelope<StoreCheckpointRecordIdentity>
{
    fn recovery_checkpoint_epoch(&self) -> u64 {
        self.record().checkpoint_epoch()
    }

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata {
        self.security_metadata().physical_metadata()
    }
}
