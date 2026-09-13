use worth_proof::TransitionOutcome;
use worth_store_contracts::StableDigest;

use super::{
    RecoveryCheckpointRecordSecurityMetadataEnvelope,
    RecoveryCheckpointRecordSecurityMetadataIdentity, RecoveryRootSecurityMetadataEnvelope,
    RecoverySecurityScopePropagationCounters, RecoverySecurityScopePropagationDenial,
    RecoverySecurityScopePropagationInput, RecoveryWalRecordSecurityMetadataEnvelope,
    RecoveryWalRecordSecurityMetadataIdentity,
};
use crate::{
    deny_drifted_store_security_scope, deny_missing_store_security_scope,
    propagate_store_security_scope, StoreSecurityMetadata, StoreSecurityScopePropagationSite,
    StoreSecurityScopePropagationWitness,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySecurityScopePropagation {
    entry_identity: StableDigest,
    wal_record_identity: RecoveryWalRecordSecurityMetadataIdentity,
    checkpoint_record_identity: RecoveryCheckpointRecordSecurityMetadataIdentity,
    witness: StoreSecurityScopePropagationWitness,
    counters: RecoverySecurityScopePropagationCounters,
}

impl RecoverySecurityScopePropagation {
    pub fn admit_required(
        wal_record: Option<&RecoveryWalRecordSecurityMetadataEnvelope>,
        checkpoint_record: Option<&RecoveryCheckpointRecordSecurityMetadataEnvelope>,
        recovery_root: Option<&RecoveryRootSecurityMetadataEnvelope>,
        entry_identity: &StableDigest,
    ) -> TransitionOutcome<Self, RecoverySecurityScopePropagationDenial> {
        let Some(wal_record) = wal_record else {
            return TransitionOutcome::denied(Self::missing_scope_denial());
        };
        let Some(checkpoint_record) = checkpoint_record else {
            return TransitionOutcome::denied(Self::missing_scope_denial());
        };
        let Some(recovery_root) = recovery_root else {
            return TransitionOutcome::denied(Self::missing_scope_denial());
        };
        Self::admit(RecoverySecurityScopePropagationInput::new(
            wal_record,
            checkpoint_record,
            recovery_root,
            entry_identity,
        ))
    }

    pub fn admit(
        input: RecoverySecurityScopePropagationInput,
    ) -> TransitionOutcome<Self, RecoverySecurityScopePropagationDenial> {
        let entry_identity = input.entry_identity().clone();
        if !input.root_artifact_present() {
            return TransitionOutcome::denied(
                RecoverySecurityScopePropagationDenial::from_store_denial(
                    deny_drifted_store_security_scope(
                        StoreSecurityScopePropagationSite::RecoveryAdmission,
                    ),
                ),
            );
        }
        let wal_checkpoint_witness = match propagate_store_security_scope(
            input.checkpoint_metadata(),
            input.wal_metadata(),
            StoreSecurityScopePropagationSite::RecoveryAdmission,
        ) {
            TransitionOutcome::Success(witness) => witness,
            TransitionOutcome::Denied(denial) => {
                return TransitionOutcome::denied(
                    RecoverySecurityScopePropagationDenial::from_store_denial(denial),
                );
            }
            TransitionOutcome::Deferred(deferred) => match deferred {},
            TransitionOutcome::Stale(stale) => match stale {},
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failed) => match failed {},
        };

        match propagate_store_security_scope(
            wal_checkpoint_witness.metadata(),
            input.root_metadata(),
            StoreSecurityScopePropagationSite::RecoveryAdmission,
        ) {
            TransitionOutcome::Success(witness) => TransitionOutcome::success(Self {
                entry_identity,
                wal_record_identity: input.wal_record_identity(),
                checkpoint_record_identity: input.checkpoint_record_identity(),
                counters: RecoverySecurityScopePropagationCounters::from_wal_checkpoint_counters(
                    wal_checkpoint_witness.counters(),
                )
                .with_root_scope_comparison(witness.counters()),
                witness,
            }),
            TransitionOutcome::Denied(denial) => TransitionOutcome::denied(
                RecoverySecurityScopePropagationDenial::from_store_denial(denial),
            ),
            TransitionOutcome::Deferred(deferred) => match deferred {},
            TransitionOutcome::Stale(stale) => match stale {},
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failed) => match failed {},
        }
    }

    pub const fn metadata(&self) -> StoreSecurityMetadata {
        self.witness.metadata()
    }

    pub const fn counters(&self) -> RecoverySecurityScopePropagationCounters {
        self.counters
    }

    pub const fn entry_identity(&self) -> &StableDigest {
        &self.entry_identity
    }

    pub const fn wal_record_identity(&self) -> RecoveryWalRecordSecurityMetadataIdentity {
        self.wal_record_identity
    }

    pub const fn checkpoint_record_identity(
        &self,
    ) -> RecoveryCheckpointRecordSecurityMetadataIdentity {
        self.checkpoint_record_identity
    }

    pub const fn witness(&self) -> StoreSecurityScopePropagationWitness {
        self.witness
    }

    fn missing_scope_denial() -> RecoverySecurityScopePropagationDenial {
        RecoverySecurityScopePropagationDenial::from_store_denial(
            deny_missing_store_security_scope(StoreSecurityScopePropagationSite::RecoveryAdmission),
        )
    }
}
