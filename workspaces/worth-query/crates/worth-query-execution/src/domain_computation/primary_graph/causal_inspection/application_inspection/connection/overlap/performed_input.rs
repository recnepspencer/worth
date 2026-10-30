//! Completed reads retain presence meaning; neither None nor a descriptor is evidence.
use super::*;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    publication::PublishedAuthoritativeFieldEffect as Effect,
    runtime::RelationalFieldPresence,
};

pub(super) struct PerformedInput<'a> {
    pub(super) entity: EntityId,
    pub(super) kind: KindId,
    pub(super) locator: &'a AspectFieldLocator,
    presence: Presence,
}
enum Presence {
    Present,
    FieldAbsent,
    AspectAbsent,
}
impl<'a> PerformedInput<'a> {
    pub(super) fn from_fact(fact: &'a Fact) -> Option<Self> {
        let (entity, kind, locator, presence) = match fact {
            Fact::Field {
                entity_id,
                kind,
                locator,
                ..
            } => (*entity_id, *kind, locator, Presence::Present),
            Fact::AbsentField {
                entity_id,
                kind,
                locator,
            } => (*entity_id, *kind, locator, Presence::FieldAbsent),
            Fact::OptionalAspectAbsent {
                entity_id,
                kind,
                locator,
            } => (*entity_id, *kind, locator, Presence::AspectAbsent),
            _ => return None,
        };
        Some(Self {
            entity,
            kind,
            locator,
            presence,
        })
    }
    pub(super) fn matches_revision(&self, presence: RelationalFieldPresence) -> bool {
        matches!(
            (&self.presence, presence),
            (Presence::Present, RelationalFieldPresence::Present)
                | (
                    Presence::FieldAbsent | Presence::AspectAbsent,
                    RelationalFieldPresence::Absent
                )
        )
    }
    pub(super) fn matches_effect(&self, effect: Effect) -> bool {
        matches!(
            (&self.presence, effect),
            (Presence::Present, Effect::Present)
                | (Presence::FieldAbsent, Effect::FieldCleared)
                | (Presence::AspectAbsent, Effect::AspectCleared)
        )
    }
}
