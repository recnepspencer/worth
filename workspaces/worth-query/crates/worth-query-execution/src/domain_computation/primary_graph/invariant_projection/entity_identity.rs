//! Projected identity traits do not impose traits on schema and entity markers.
use super::{Arc, EntityId, KindId, PhantomData};
pub struct WorthQueryInvariantEntityIdentity<Schema, Entity> {
    pub(super) entity_id: EntityId,
    pub(super) kind: KindId,
    pub(super) entity: Arc<str>,
    pub(super) authority_identity: u64,
    pub(super) phase:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    pub(super) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<S, E> Clone for WorthQueryInvariantEntityIdentity<S, E> {
    fn clone(&self) -> Self {
        Self {
            entity_id: self.entity_id,
            kind: self.kind,
            entity: Arc::clone(&self.entity),
            authority_identity: self.authority_identity,
            phase: self.phase,
            _marker: PhantomData,
        }
    }
}
impl<S, E> std::fmt::Debug for WorthQueryInvariantEntityIdentity<S, E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryInvariantEntityIdentity")
            .field("entity_id", &self.entity_id)
            .field("kind", &self.kind)
            .field("entity", &self.entity)
            .field("authority_identity", &self.authority_identity)
            .field("phase", &self.phase)
            .field("_marker", &self._marker)
            .finish()
    }
}
