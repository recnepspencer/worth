//! Bringing one unchanged record back under judgement.
//!
//! Every other entity intent here exists to change a record. This one exists
//! because a candidate can change the meaning records are judged under without
//! touching the records themselves, and rules only ever see what a candidate
//! touches. Applying it marks the record's slot touched and stops: the stored
//! state, its aspect version and its lineage are left exactly as they were, so
//! the only thing the record gains is a rule's attention.

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::ensure_entity_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{CommitConflict, RevalidateEntityIntent};

pub(super) fn apply(
    intent: &RevalidateEntityIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    workspace.with_context(|context| {
        ensure_entity_target_is_current(context.state, intent.entity_id)?;
        context
            .state
            .mark_entity_slot_touched(intent.entity_id.partition_id, intent.entity_id.slot_index());
        Ok::<(), CommitConflict>(())
    })?;
    Ok(MutationOutcome::record_revalidated())
}
