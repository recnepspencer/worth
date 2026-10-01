//! Rebind the selected V2 checkpoint-source roster at the one-shot Serving
//! boundary. The retained media fingerprint separately covers every head
//! block and control frame reread by Store before this seal was minted.

use worth_store_physical_format::{
    ReleaseCheckpointCertificateV1, ReleaseCustodyHeadRosterDigestV1,
};
use worth_store_recovery_physics::VerifiedSelectedReleaseHeadCustodyV2;

use super::*;

impl RecoveredPhysicalCheckpointCustody {
    pub(super) fn verify_head_v2(
        &self,
        claim: &VerifiedSelectedReleaseHeadCustodyV2,
        store: StableStoreIdentity,
        root: &DurablePhysicalRootManifest,
        actual_sha256: [u8; 32],
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), RecoveredCheckpointCustodyDenial> {
        let denial = RecoveredCheckpointCustodyDenial::SelectedCheckpointMismatch;
        let source = claim.checkpoint_source_root();
        let checkpoint = claim.checkpoint();
        let accumulator = claim.accumulator_v2();
        let base = accumulator.base();
        let source_sha: [u8; 32] = Sha256::digest(source.encode(format)).into();
        if claim.selected_root() != root
            || claim.selected_root_sha256() != actual_sha256
            // An ordinary authenticated publication may advance the selected
            // root without changing checkpoint-source head custody. Store's
            // retained fingerprint covers both actual root/control closures.
            // A changed head reference/frontier still needs V14 replay custody.
            || root.tree_identity() != source.tree_identity()
            || root.tier_epoch_anchor() != source.tier_epoch_anchor()
            || root.generation() < source.generation()
            || (root.generation() == source.generation() && root != source)
            || root.release_custody_head_root() != source.release_custody_head_root()
            || root.next_release_custody_head_block() != source.next_release_custody_head_block()
            || source_sha != claim.source_root_sha256()
            || checkpoint.source().identity().store_identity() != store
            || checkpoint.source().root().generation() != source.generation()
            || checkpoint.source().root().tree_identity() != source.tree_identity()
            || base.checkpoint() != checkpoint.source().identity()
            || base.root_generation() != source.generation()
            || base.root_sha256() != source_sha
        {
            return Err(denial);
        }
        let mut roster = ReleaseCustodyHeadRosterDigestV1::new(
            source.release_custody_head_root(),
            accumulator.head_count(),
        );
        for entry in claim.selected_heads() {
            roster.push(*entry).map_err(|_| denial)?;
        }
        if roster.finish() != (accumulator.head_count(), accumulator.head_roster_digest()) {
            return Err(denial);
        }
        let mut expected = claim
            .batches()
            .iter()
            .copied()
            .map(ReleaseCheckpointCertificateV1::Batch)
            .chain(std::iter::once(
                ReleaseCheckpointCertificateV1::AccumulatorV2(accumulator),
            ));
        let mut observed_count = 0_usize;
        for frame in checkpoint.certificate_records() {
            let (kind, payload) = decode_checkpoint_certificate(frame).map_err(|_| denial)?;
            if kind == CheckpointCertificateKind::ReleasedDrop {
                let observed =
                    ReleaseCheckpointCertificateV1::decode(payload).map_err(|_| denial)?;
                if expected.next() != Some(observed) {
                    return Err(denial);
                }
                observed_count += 1;
            }
        }
        if expected.next().is_some()
            || observed_count != usize::from(claim.release_certificate_record_count())
        {
            return Err(denial);
        }
        Ok(())
    }
}
