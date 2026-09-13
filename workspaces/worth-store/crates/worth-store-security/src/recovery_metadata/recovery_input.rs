use worth_store_contracts::StableDigest;

use super::{
    RecoveryCheckpointRecordSecurityMetadataSource, RecoveryWalRecordSecurityMetadataSource,
};
use crate::{
    StoreAdmittedSecurityScope, StoreKeyVersionPosture, StoreLegacySecurityPosture,
    StoreSecurityMetadata,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryWalRecordSecurityMetadataIdentity {
    sequence: u64,
}

impl RecoveryWalRecordSecurityMetadataIdentity {
    pub const fn new(sequence: u64) -> Self {
        Self { sequence }
    }

    pub fn from_store_wal_record<R>(record: &R) -> Self
    where
        R: RecoveryWalRecordSecurityMetadataSource,
    {
        Self::new(record.recovery_sequence())
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCheckpointRecordSecurityMetadataIdentity {
    checkpoint_epoch: u64,
}

impl RecoveryCheckpointRecordSecurityMetadataIdentity {
    pub const fn new(checkpoint_epoch: u64) -> Self {
        Self { checkpoint_epoch }
    }

    pub fn from_store_checkpoint_record<R>(record: &R) -> Self
    where
        R: RecoveryCheckpointRecordSecurityMetadataSource,
    {
        Self::new(record.recovery_checkpoint_epoch())
    }

    pub const fn checkpoint_epoch(self) -> u64 {
        self.checkpoint_epoch
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryWalRecordSecurityMetadataEnvelope {
    identity: RecoveryWalRecordSecurityMetadataIdentity,
    security_metadata: StoreSecurityMetadata,
}

impl RecoveryWalRecordSecurityMetadataEnvelope {
    pub fn from_wal_record_envelope<R>(record: &R) -> Self
    where
        R: RecoveryWalRecordSecurityMetadataSource,
    {
        Self::new(
            RecoveryWalRecordSecurityMetadataIdentity::from_store_wal_record(record),
            record.recovery_security_metadata(),
        )
    }

    pub fn from_admitted_scope(
        identity: RecoveryWalRecordSecurityMetadataIdentity,
        admitted_scope: &StoreAdmittedSecurityScope,
        key_version_posture: StoreKeyVersionPosture,
        legacy_posture: StoreLegacySecurityPosture,
    ) -> Self {
        Self::new(
            identity,
            StoreSecurityMetadata::from_current_security_scope(
                admitted_scope.witnesses(),
                key_version_posture,
                legacy_posture,
            ),
        )
    }

    const fn new(
        identity: RecoveryWalRecordSecurityMetadataIdentity,
        security_metadata: StoreSecurityMetadata,
    ) -> Self {
        Self {
            identity,
            security_metadata,
        }
    }

    pub const fn identity(&self) -> RecoveryWalRecordSecurityMetadataIdentity {
        self.identity
    }

    pub const fn security_metadata(&self) -> StoreSecurityMetadata {
        self.security_metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCheckpointRecordSecurityMetadataEnvelope {
    identity: RecoveryCheckpointRecordSecurityMetadataIdentity,
    security_metadata: StoreSecurityMetadata,
}

impl RecoveryCheckpointRecordSecurityMetadataEnvelope {
    pub fn from_checkpoint_record_envelope<R>(record: &R) -> Self
    where
        R: RecoveryCheckpointRecordSecurityMetadataSource,
    {
        Self::new(
            RecoveryCheckpointRecordSecurityMetadataIdentity::from_store_checkpoint_record(record),
            record.recovery_security_metadata(),
        )
    }

    pub fn from_admitted_scope(
        identity: RecoveryCheckpointRecordSecurityMetadataIdentity,
        admitted_scope: &StoreAdmittedSecurityScope,
        key_version_posture: StoreKeyVersionPosture,
        legacy_posture: StoreLegacySecurityPosture,
    ) -> Self {
        Self::new(
            identity,
            StoreSecurityMetadata::from_current_security_scope(
                admitted_scope.witnesses(),
                key_version_posture,
                legacy_posture,
            ),
        )
    }

    const fn new(
        identity: RecoveryCheckpointRecordSecurityMetadataIdentity,
        security_metadata: StoreSecurityMetadata,
    ) -> Self {
        Self {
            identity,
            security_metadata,
        }
    }

    pub const fn identity(&self) -> RecoveryCheckpointRecordSecurityMetadataIdentity {
        self.identity
    }

    pub const fn security_metadata(&self) -> StoreSecurityMetadata {
        self.security_metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRootSecurityMetadataEnvelope {
    entry_identity: StableDigest,
    security_metadata: StoreSecurityMetadata,
}

impl RecoveryRootSecurityMetadataEnvelope {
    pub fn from_recovery_entry(
        entry_identity: &StableDigest,
        admitted_scope: &StoreAdmittedSecurityScope,
        key_version_posture: StoreKeyVersionPosture,
        legacy_posture: StoreLegacySecurityPosture,
    ) -> Self {
        Self::new(
            entry_identity.clone(),
            StoreSecurityMetadata::from_current_security_scope(
                admitted_scope.witnesses(),
                key_version_posture,
                legacy_posture,
            ),
        )
    }

    const fn new(entry_identity: StableDigest, security_metadata: StoreSecurityMetadata) -> Self {
        Self {
            entry_identity,
            security_metadata,
        }
    }

    pub const fn entry_identity(&self) -> &StableDigest {
        &self.entry_identity
    }

    pub const fn security_metadata(&self) -> StoreSecurityMetadata {
        self.security_metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySecurityScopePropagationInput {
    wal_record_identity: RecoveryWalRecordSecurityMetadataIdentity,
    checkpoint_record_identity: RecoveryCheckpointRecordSecurityMetadataIdentity,
    root_artifact_present: bool,
    wal_metadata: StoreSecurityMetadata,
    checkpoint_metadata: StoreSecurityMetadata,
    root_metadata: StoreSecurityMetadata,
    entry_identity: StableDigest,
}

impl RecoverySecurityScopePropagationInput {
    pub fn new(
        wal_record: &RecoveryWalRecordSecurityMetadataEnvelope,
        checkpoint_record: &RecoveryCheckpointRecordSecurityMetadataEnvelope,
        recovery_root: &RecoveryRootSecurityMetadataEnvelope,
        entry_identity: &StableDigest,
    ) -> Self {
        Self {
            wal_record_identity: wal_record.identity(),
            checkpoint_record_identity: checkpoint_record.identity(),
            root_artifact_present: recovery_root.entry_identity() == entry_identity,
            wal_metadata: wal_record.security_metadata(),
            checkpoint_metadata: checkpoint_record.security_metadata(),
            root_metadata: recovery_root.security_metadata(),
            entry_identity: entry_identity.clone(),
        }
    }

    pub const fn wal_record_identity(&self) -> RecoveryWalRecordSecurityMetadataIdentity {
        self.wal_record_identity
    }

    pub const fn checkpoint_record_identity(
        &self,
    ) -> RecoveryCheckpointRecordSecurityMetadataIdentity {
        self.checkpoint_record_identity
    }

    pub const fn root_artifact_present(&self) -> bool {
        self.root_artifact_present
    }

    pub const fn wal_metadata(&self) -> StoreSecurityMetadata {
        self.wal_metadata
    }

    pub const fn checkpoint_metadata(&self) -> StoreSecurityMetadata {
        self.checkpoint_metadata
    }

    pub const fn root_metadata(&self) -> StoreSecurityMetadata {
        self.root_metadata
    }

    pub const fn entry_identity(&self) -> &StableDigest {
        &self.entry_identity
    }
}
