use std::marker::PhantomData;

use worth_relational::facade::identity::EntityId;

use super::WorthQueryApplicationOutputPosture;

/// One typed member of a declared output-role family, as a commit bound it.
pub struct WorthQueryApplicationOutputFamilyEntry<'correspondence, Entity> {
    pub(crate) role: &'correspondence str,
    pub(crate) posture: WorthQueryApplicationOutputPosture,
    pub(crate) entity_id: EntityId,
    pub(crate) _marker: PhantomData<fn() -> Entity>,
}

impl<Entity> Copy for WorthQueryApplicationOutputFamilyEntry<'_, Entity> {}

impl<Entity> Clone for WorthQueryApplicationOutputFamilyEntry<'_, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Entity> std::fmt::Debug for WorthQueryApplicationOutputFamilyEntry<'_, Entity> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationOutputFamilyEntry")
            .field("role", &self.role)
            .field("posture", &self.posture)
            .field("entity_id", &self.entity_id)
            .finish()
    }
}

impl<Entity> PartialEq for WorthQueryApplicationOutputFamilyEntry<'_, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role
            && self.posture == other.posture
            && self.entity_id == other.entity_id
    }
}

impl<Entity> Eq for WorthQueryApplicationOutputFamilyEntry<'_, Entity> {}

impl<Entity> WorthQueryApplicationOutputFamilyEntry<'_, Entity> {
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

/// Crate-boundary internal port for `WorthQueryApplicationOutputFamilyEntry`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ApplicationOutputFamilyEntryInternalPort<'correspondence, Entity>: Sized {
    fn from_parts(
        role: &'correspondence str,
        posture: WorthQueryApplicationOutputPosture,
        entity_id: EntityId,
    ) -> Self;
}

impl<'correspondence, Entity> ApplicationOutputFamilyEntryInternalPort<'correspondence, Entity>
    for WorthQueryApplicationOutputFamilyEntry<'correspondence, Entity>
{
    fn from_parts(
        role: &'correspondence str,
        posture: WorthQueryApplicationOutputPosture,
        entity_id: EntityId,
    ) -> Self {
        Self {
            role,
            posture,
            entity_id,
            _marker: PhantomData,
        }
    }
}
