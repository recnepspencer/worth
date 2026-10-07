//! Completed checkpoint-to-source edges and the final pending V14 head fold.
//! Storage is prepared before effects; publication is admitted only afterward.

use worth_store_physical_format::ReleaseCustodyHeadRosterDigestV1;

use super::{
    EffectiveReleaseHeadDenial as Denial, VerifiedEffectiveReleaseHeadRosterV14,
    VerifiedPendingWalReleaseCustody,
};

#[path = "ordered/preparation.rs"]
mod preparation;
#[path = "ordered/validation.rs"]
mod validation;

impl VerifiedEffectiveReleaseHeadRosterV14 {
    pub fn admit_ordered_pending(
        claim: &mut VerifiedPendingWalReleaseCustody,
        maximum_entries: u64,
        maximum_retained_bytes: u64,
    ) -> Result<Self, Denial> {
        if claim.published_root.is_none() || claim.verified_transition.is_none() {
            return Err(Denial::Source);
        }
        let prepared = claim
            .prepared_effective_heads
            .as_ref()
            .ok_or(Denial::Source)?;
        if prepared.ordered_replays.is_empty() {
            return Err(Denial::Source);
        }
        let basis = validation::validate(claim, &prepared.ordered_replays, maximum_entries)?;
        let retained_bytes = basis.retained_bytes::<Self>(&prepared.ordered_replays)?;
        if retained_bytes > maximum_retained_bytes {
            return Err(Denial::BoundExceeded);
        }
        if prepared.checkpoint_source_heads != basis.source_heads
            || prepared.effective_heads.len() > basis.result_capacity()
            || !validation::matches_exact_fold(claim, prepared, basis.source_heads)
        {
            return Err(Denial::Mutation);
        }
        let pending = claim
            .selected_head_replay
            .as_ref()
            .ok_or(Denial::MissingReplay)?;
        let published = claim.published_root.as_ref().ok_or(Denial::Source)?;
        if published.release_custody_head_root() != Some(pending.result_root())
            || published.next_release_custody_head_block() != pending.result_next_block()
        {
            return Err(Denial::Source);
        }
        let mut digest =
            ReleaseCustodyHeadRosterDigestV1::new(Some(pending.result_root()), maximum_entries);
        for entry in &prepared.effective_heads {
            digest.push(*entry).map_err(|_| Denial::Mutation)?;
        }
        let (_, effective_digest) = digest.finish();
        let effective_root_frame_sha256 = claim.published_root_sha256.ok_or(Denial::Source)?;
        let checkpoint_ref = basis.checkpoint_ref;
        let checkpoint_frontier = basis.checkpoint_frontier;
        let effective_root = pending.result_root();
        let effective_next_block = pending.result_next_block();
        let prepared = claim
            .prepared_effective_heads
            .take()
            .ok_or(Denial::Source)?;
        Ok(Self {
            checkpoint_source_heads: prepared.checkpoint_source_heads,
            effective_heads: prepared.effective_heads,
            checkpoint_source_root: checkpoint_ref,
            checkpoint_source_next_block: checkpoint_frontier,
            effective_root,
            effective_next_block,
            effective_digest,
            effective_root_frame_sha256,
            ordered_replays: prepared.ordered_replays,
            retained_bytes,
        })
    }
}
