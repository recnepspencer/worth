use worth_relational::facade::identity::EntityId;

use super::super::invariant_projection::WorthQueryRealizedProjectionScope;

/// Instance authority for identities that may enter one application read set.
pub struct WorthQueryApplicationReadScope {
    root: EntityId,
    projected: Option<WorthQueryRealizedProjectionScope>,
}

impl WorthQueryApplicationReadScope {
    pub const fn root_only(root: EntityId) -> Self {
        Self {
            root,
            projected: None,
        }
    }

    pub const fn projected(root: EntityId, projected: WorthQueryRealizedProjectionScope) -> Self {
        Self {
            root,
            projected: Some(projected),
        }
    }

    pub fn admits(&self, entity_id: EntityId) -> bool {
        entity_id == self.root
            || self
                .projected
                .as_ref()
                .is_some_and(|scope| scope.contains(entity_id))
    }
}
