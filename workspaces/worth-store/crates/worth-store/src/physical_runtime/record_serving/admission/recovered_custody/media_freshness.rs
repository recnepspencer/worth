//! Native media freshness advances custody through WAL, checkpoint, head,
//! and selected-control reads before Serving admission.

use super::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    QualifiedFilesystemMedia, RecoveredCheckpointCustodyDenial, RecoveredCheckpointCustodyEvidence,
    StableStoreIdentity, VerifiedRecoveredCheckpointCustody,
};
use crate::physical_runtime::{
    record_serving::RecordBootstrapDenial, PhysicalRecoveryReadAllocation,
};

/// Only the consuming fresh-media read below can advance one-shot custody.
pub(in crate::physical_runtime) struct WalVerifiedRecoveredCustodyEvidence {
    evidence: RecoveredCheckpointCustodyEvidence,
}

pub(in crate::physical_runtime) struct FundedMediaVerifiedRecoveredCustodyEvidence {
    evidence: RecoveredCheckpointCustodyEvidence,
}

impl RecoveredCheckpointCustodyEvidence {
    pub(in crate::physical_runtime) fn verify_wal_for_serving(
        self,
        media: &QualifiedFilesystemMedia,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<WalVerifiedRecoveredCustodyEvidence, RecordBootstrapDenial> {
        if self.store != store || self.root != *root {
            return Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch);
        }
        if !self
            .selected_wal
            .matches_serving_media(media, window)
            .map_err(RecordBootstrapDenial::RecoveredWalRead)?
        {
            return Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch);
        }
        Ok(WalVerifiedRecoveredCustodyEvidence { evidence: self })
    }
}

impl WalVerifiedRecoveredCustodyEvidence {
    pub(in crate::physical_runtime) fn verify_selected_media_for_serving(
        self,
        media: &QualifiedFilesystemMedia,
        format: PhysicalRecordFormatDeclaration,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<FundedMediaVerifiedRecoveredCustodyEvidence, RecordBootstrapDenial> {
        if let Some(checkpoint) = self.evidence.checkpoint_ownership.checkpoint() {
            if !window.owns_checkpoint(checkpoint) {
                return Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch);
            }
        }
        self.evidence
            .verify_funded_current_checkpoint(media, window)?;
        let controls = self
            .evidence
            .selected_controls
            .as_ref()
            .ok_or(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
        let required_heads_present = match (
            &self.evidence.head_v2,
            &self.evidence.pending_wal_release,
            &self.evidence.historical_release,
        ) {
            (Some(_), None, None) => controls.has_selected_head_walk(),
            (None, Some(_), None) => controls.has_pending_head_walks(),
            (None, None, Some(_)) => controls.has_completed_history_head_walks(),
            (None, None, None) => controls.has_no_head_walk(),
            _ => false,
        };
        if !required_heads_present {
            return Err(RecordBootstrapDenial::RecoveredHeadWitnessPostureMismatch);
        }
        if !controls.verify_funded_heads_for_serving(media, format, window)? {
            return Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch);
        }
        if self.evidence.historical_release.is_some()
            && !controls.verify_funded_completed_raw_for_serving(media, format, window)?
        {
            return Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch);
        }
        Ok(FundedMediaVerifiedRecoveredCustodyEvidence {
            evidence: self.evidence,
        })
    }
}

impl FundedMediaVerifiedRecoveredCustodyEvidence {
    pub(in crate::physical_runtime) fn verify_for_serving(
        self,
        media: &QualifiedFilesystemMedia,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        free_space: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<VerifiedRecoveredCheckpointCustody, RecoveredCheckpointCustodyDenial> {
        self.evidence
            .verify_controls_for_open(media, store, root, free_space, format)?;
        Ok(VerifiedRecoveredCheckpointCustody {
            seal: self.evidence,
        })
    }
}
