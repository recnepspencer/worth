use std::marker::PhantomData;

use worth_relational::facade::identity::EntityId;

use super::WorthQueryApplicationOutputPosture;

/// One typed member of a sealed output-role family.
pub struct WorthQueryApplicationOutputFamilyEntry<'correspondence, Binding, Entity> {
    pub(super) role: &'correspondence str,
    pub(super) posture: WorthQueryApplicationOutputPosture,
    pub(super) entity_id: EntityId,
    pub(super) _marker: PhantomData<fn() -> (Binding, Entity)>,
}

impl<Binding, Entity> Copy for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {}

impl<Binding, Entity> Clone for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Binding, Entity> std::fmt::Debug
    for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationOutputFamilyEntry")
            .field("role", &self.role)
            .field("posture", &self.posture)
            .field("entity_id", &self.entity_id)
            .finish()
    }
}

impl<Binding, Entity> PartialEq for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role
            && self.posture == other.posture
            && self.entity_id == other.entity_id
    }
}

impl<Binding, Entity> Eq for WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {}

impl<Binding, Entity> WorthQueryApplicationOutputFamilyEntry<'_, Binding, Entity> {
    pub const fn role(&self) -> &str {
        self.role
    }

    pub const fn posture(&self) -> WorthQueryApplicationOutputPosture {
        self.posture
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}
