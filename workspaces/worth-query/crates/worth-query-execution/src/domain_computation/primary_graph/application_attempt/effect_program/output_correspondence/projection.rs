use std::marker::PhantomData;

use worth_relational::facade::identity::EntityId;

/// The committed entity behind one declared output role, projected from a
/// `WorthQueryApplicationOutputCorrespondence` with its contract, entity type
/// and posture checked.
///
/// Get one from the correspondence's `entity` or `member`; read the identity
/// with `entity_id`. It describes what the commit produced and grants nothing.
pub struct WorthQueryApplicationOutputEntity<Entity, Action> {
    entity_id: EntityId,
    _marker: PhantomData<fn() -> (Entity, Action)>,
}

impl<Entity, Action> WorthQueryApplicationOutputEntity<Entity, Action> {
    pub(super) const fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            _marker: PhantomData,
        }
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}
