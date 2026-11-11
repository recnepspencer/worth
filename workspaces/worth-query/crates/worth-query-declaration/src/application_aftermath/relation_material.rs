//! Historical relation endpoints are descriptions, never current graph handles.
use worth_foundational::facade::EntityId;
/// Before and performed endpoint pairs for one relation changed by the original actor.
/// The typed locus supplies the relation kind and endpoint entity roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCorrectionRelationMaterial {
    before: Option<(EntityId, EntityId)>,
    after: Option<(EntityId, EntityId)>,
}
impl ApplicationCorrectionRelationMaterial {
    /// Descriptive material may be authored or decoded; commit admission independently
    /// compares the actual inverse with the source occurrence's sealed original.
    pub const fn from_historical_endpoints(
        before: Option<(EntityId, EntityId)>,
        after: Option<(EntityId, EntityId)>,
    ) -> Self {
        Self { before, after }
    }
    pub const fn before(&self) -> Option<(EntityId, EntityId)> {
        self.before
    }
    pub const fn after(&self) -> Option<(EntityId, EntityId)> {
        self.after
    }
}
