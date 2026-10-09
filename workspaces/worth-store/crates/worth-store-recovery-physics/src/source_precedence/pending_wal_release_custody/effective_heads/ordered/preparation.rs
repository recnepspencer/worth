//! Pre-effect ordered storage preparation; it never asserts publication.

use worth_store_physical_format::ReleaseCustodyHeadMutationV1;

use super::super::{
    apply_entry, retained_storage, EffectiveReleaseHeadDenial as Denial,
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedPendingWalReleaseCustody,
};
use super::validation;
use crate::VerifiedOrderedReleasedHeadReplayV14;

impl VerifiedPendingWalReleaseCustody {
    /// Charge only the two newly allocated roster backings. The caller already
    /// owns and charges the addressed attachments passed into this operation.
    pub fn prepare_ordered_effective_heads(
        &mut self,
        attachments: Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
        maximum_entries: u64,
        maximum_retained_bytes: u64,
        maximum_additional_resident_bytes: u64,
    ) -> Result<u64, Denial> {
        if self.prepared_effective_heads.is_some()
            || self.published_root.is_some()
            || self.verified_transition.is_some()
        {
            return Err(Denial::Source);
        }
        let basis = validation::validate(self, &attachments, maximum_entries)?;
        if basis.retained_bytes::<VerifiedEffectiveReleaseHeadRosterV14>(&attachments)?
            > maximum_retained_bytes
        {
            return Err(Denial::BoundExceeded);
        }
        let (mut source_copy, mut effective) = retained_storage::reserve_rosters(
            basis.source_heads.len(),
            basis.result_capacity(),
            maximum_additional_resident_bytes,
        )?;
        source_copy.extend_from_slice(basis.source_heads);
        effective.extend_from_slice(basis.source_heads);
        for (_, attachment) in &attachments {
            let ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior,
                next,
            } = attachment.replay().effect().mutation()
            else {
                return Err(Denial::Mutation);
            };
            apply_entry(&mut effective, expected_prior, next, maximum_entries)?;
        }
        let pending = self
            .selected_head_replay
            .as_ref()
            .ok_or(Denial::MissingReplay)?;
        let ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior,
            next,
        } = pending.effect().mutation()
        else {
            return Err(Denial::Mutation);
        };
        apply_entry(&mut effective, expected_prior, next, maximum_entries)?;
        let prepared = retained_storage::PreparedEffectiveHeadRosterV14 {
            checkpoint_source_heads: source_copy,
            effective_heads: effective,
            ordered_replays: attachments,
        };
        let backing = prepared
            .new_roster_backing_bytes()
            .ok_or(Denial::BoundExceeded)?;
        self.prepared_effective_heads = Some(prepared);
        Ok(backing)
    }
}
