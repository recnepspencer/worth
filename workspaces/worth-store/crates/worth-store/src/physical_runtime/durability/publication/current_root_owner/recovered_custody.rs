//! Recovered custody is prepared before Serving progression. Installation
//! moves an admitted ledger, whose capture reservation Serving entry funded,
//! and never performs fallible allocation.

use super::{certificate_capacity, release_capacity, PhysicalCurrentRootOwner};
use crate::physical_runtime::record_serving::VerifiedRecoveredCheckpointCustody;
#[cfg(feature = "recovery-runtime-owner")]
use crate::physical_runtime::recovery_residency::StoreRejoinResidentLedger;

pub(in crate::physical_runtime) struct PreparedRecoveredCheckpointCustody {
    source: VerifiedRecoveredCheckpointCustody,
    selected_root: worth_store_physical_format::DurablePhysicalRootManifest,
    release_ledger: release_capacity::ReleaseLedgerState,
    checkpoint_custody: certificate_capacity::CheckpointCustodyState,
}

impl PreparedRecoveredCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn prepare(
        source: VerifiedRecoveredCheckpointCustody,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, release_capacity::RecoveredReleaseLedgerDenial> {
        use release_capacity::RecoveredReleaseLedgerDenial as Denial;
        let (head_v2, no_release, pending, effective, historical, tier) =
            source.seal().verified_basis();
        let selected_root = head_v2
            .map(|claim| claim.selected_root())
            .or_else(|| no_release.map(|claim| claim.selected_root()))
            .or_else(|| pending.and_then(|claim| claim.published_root()))
            .or_else(|| historical.map(|claim| claim.selected_root()))
            .ok_or(Denial::SelectedFactMismatch)?
            .clone();
        let release_ledger = match (head_v2, no_release, pending, historical) {
            (Some(verified), None, None, None) => {
                release_capacity::ReleaseLedgerState::from_verified_v2_source(
                    verified.accumulator_v2(),
                    verified
                        .checkpoint_source_root()
                        .release_custody_head_root(),
                    verified.selected_heads().iter().copied(),
                    resident,
                )?
            }
            (None, Some(verified), None, None) => {
                release_capacity::ReleaseLedgerState::from_verified_no_release(verified)
            }
            (None, None, Some(verified), None) => {
                release_capacity::ReleaseLedgerState::from_verified_pending_wal(
                    verified,
                    effective.ok_or(Denial::SelectedFactMismatch)?,
                    resident,
                )?
            }
            (None, None, None, Some(verified)) => {
                release_capacity::ReleaseLedgerState::from_verified_ordered_historical(
                    verified,
                    effective.ok_or(Denial::SelectedFactMismatch)?,
                    resident,
                )?
            }
            _ => return Err(Denial::SelectedFactMismatch),
        };
        let checkpoint_custody = match tier {
            Some(verified) => {
                if verified.selected_root() != &selected_root
                    || selected_root.tier_epoch_anchor() != Some(verified.tier_epoch_anchor())
                    || verified.free_header().tier_epoch_start()
                        != Some(verified.tier_epoch_start())
                {
                    return Err(Denial::SelectedFactMismatch);
                }
                certificate_capacity::CheckpointCustodyState::CertifiedTier(
                    certificate_capacity::SealedTierEpochCustodyBasis::from_verified_selected(
                        verified,
                    ),
                )
            }
            None if selected_root.tier_epoch_anchor().is_none() => {
                certificate_capacity::CheckpointCustodyState::VerifiedLegacyNoCertificates
            }
            None => return Err(Denial::SelectedFactMismatch),
        };
        Ok(Self {
            source,
            selected_root,
            release_ledger,
            checkpoint_custody,
        })
    }

    pub(super) fn release_ledger_mut(&mut self) -> &mut release_capacity::ReleaseLedgerState {
        &mut self.release_ledger
    }

    pub(in crate::physical_runtime) fn pending_wal_release(
        &self,
    ) -> Option<&worth_store_recovery_physics::VerifiedPendingWalReleaseCustody> {
        self.source.seal().pending_wal_release()
    }
}

impl PhysicalCurrentRootOwner {
    pub(super) fn install_recovered_checkpoint_custody(
        &self,
        recovered: PreparedRecoveredCheckpointCustody,
    ) {
        let PreparedRecoveredCheckpointCustody {
            source,
            selected_root,
            release_ledger,
            checkpoint_custody,
        } = recovered;
        let mut state = self.lock_publication_state();
        assert_eq!(selected_root, state.current_root);
        if let Some(tier) = source.seal().verified_basis().5 {
            assert_eq!(tier.free_header(), &state.free_space);
        }
        // Drop the old proof backing only after its conversion has completed.
        drop(source);
        state.release_ledger = release_ledger;
        state.checkpoint_custody = checkpoint_custody;
    }
}
