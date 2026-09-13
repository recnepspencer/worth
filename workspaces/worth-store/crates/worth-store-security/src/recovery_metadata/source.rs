use crate::StoreSecurityMetadata;

/// Supplies only the durable facts needed to build a recovery security envelope.
///
/// The trait keeps the security owner independent of the WAL crate while allowing the
/// WAL owner to expose its already-admitted record envelopes at this boundary.
pub trait RecoveryWalRecordSecurityMetadataSource {
    fn recovery_sequence(&self) -> u64;

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata;
}

/// Supplies only the durable facts needed to build a checkpoint security envelope.
pub trait RecoveryCheckpointRecordSecurityMetadataSource {
    fn recovery_checkpoint_epoch(&self) -> u64;

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata;
}
