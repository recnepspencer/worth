//! Selected C.9 WAL fate for the one-time tier activation.

use worth_store_physical_format::TierEpochWalFrameWitnessV1;
use worth_store_recovery_physics::{
    SelectedTierEpochCustodySource, VerifiedSelectedTierEpochCustody,
};

use super::super::{wal_inventory::AdmittedWalInventory, SelectedMediaRejoinDenial as Denial};
use crate::physical_runtime::StoreRecoveryBindingFreshnessSample;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify(
    inventory: &AdmittedWalInventory,
    claim: &VerifiedSelectedTierEpochCustody,
) -> Result<(), Denial> {
    let intent = claim.intent_frame();
    let completed = claim.completed_frame();
    let cutoff = claim
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    let intent_matches = exact_overlap(inventory, intent)?;
    let completed_matches = exact_overlap(inventory, completed)?;
    match claim.source() {
        SelectedTierEpochCustodySource::SelectedWalPair
            if intent_matches
                && completed_matches
                && intent.lsn_start() >= cutoff
                && completed.lsn_start() >= cutoff =>
        {
            Ok(())
        }
        // The selected tag-6 checkpoint is the durable custody ratchet after
        // compaction. Retained overlaps must still match its exact witnesses.
        SelectedTierEpochCustodySource::SelectedCheckpointCertificate
            if claim.certificate().is_some() && completed.lsn_end_exclusive() <= cutoff =>
        {
            Ok(())
        }
        _ => Err(Denial::WalFate),
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_sample(
    sample: &StoreRecoveryBindingFreshnessSample,
    claim: &VerifiedSelectedTierEpochCustody,
) -> Result<(), Denial> {
    let Some(observed) = sample.tier_epoch_activation() else {
        return matches!(
            claim.source(),
            SelectedTierEpochCustodySource::SelectedCheckpointCertificate
        )
        .then_some(())
        .ok_or(Denial::WalFate);
    };
    let intent = claim.intent_frame();
    let completed = claim.completed_frame();
    let exact = observed.intent() == claim.intent()
        && observed.intent_range().start().get() == intent.lsn_start()
        && observed.intent_range().end_exclusive().get() == intent.lsn_end_exclusive()
        && observed.completion_range().is_some_and(|range| {
            range.start().get() == completed.lsn_start()
                && range.end_exclusive().get() == completed.lsn_end_exclusive()
        });
    exact.then_some(()).ok_or(Denial::WalFate)
}

fn exact_overlap(
    inventory: &AdmittedWalInventory,
    witness: TierEpochWalFrameWitnessV1,
) -> Result<bool, Denial> {
    let mut found = false;
    for frame in inventory.frames() {
        if frame.lsn_start() < witness.lsn_end_exclusive() && witness.lsn_start() < frame.lsn_end()
        {
            if found
                || frame.lsn_start() != witness.lsn_start()
                || frame.lsn_end() != witness.lsn_end_exclusive()
                || frame.identity_digest() != witness.identity_digest()
                || frame.payload_digest() != witness.payload_digest()
            {
                return Err(Denial::WalFate);
            }
            found = true;
        }
    }
    Ok(found)
}
