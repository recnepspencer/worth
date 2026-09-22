//! Whether one entity intent claims a write locus or only a read locus.
//!
//! Two lanes need this answer: staging admission, which bounds everything a
//! batch will touch, and the overlay, which records what each staged intent
//! claims. Answering it twice is how the two drift, so the decision lives here
//! and both consume it. An intent that authors the record claims a write; a
//! revalidation demand claims a read, because it only brings the record back
//! under judgement and leaves it exactly as it was.
//!
//! Carrying the locus and the record id together in one value is the point. A
//! separate "which record does this intent name" helper would hand a write
//! path an id for a demand, and nothing but arm order would stop it being
//! recorded as authored.

use crate::identity::data::EntityId;
use crate::transactions::data::EntityMutationIntent;

/// The one locus an entity intent claims, named together with its record.
pub(super) enum EntityIntentLocus {
    /// The intent authors this record.
    Write(EntityId),
    /// The intent only observes this record.
    Read(EntityId),
}

pub(super) const fn entity_intent_locus(intent: &EntityMutationIntent) -> EntityIntentLocus {
    match intent {
        EntityMutationIntent::UpdateFields(intent) => EntityIntentLocus::Write(intent.entity_id),
        EntityMutationIntent::ApplyAspectPatch(intent) => {
            EntityIntentLocus::Write(intent.entity_id)
        }
        EntityMutationIntent::Replace(intent) => EntityIntentLocus::Write(intent.entity_id),
        EntityMutationIntent::Delete(intent) => EntityIntentLocus::Write(intent.entity_id),
        EntityMutationIntent::Revalidate(intent) => EntityIntentLocus::Read(intent.entity_id),
    }
}
