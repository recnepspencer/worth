//! Projected identity traits do not impose traits on schema and entity markers.
use super::{Arc, EntityId, KindId, PhantomData};
pub struct WorthQueryInvariantEntityIdentity<Schema, Entity> {
    pub(crate) entity_id: EntityId,
    pub(crate) kind: KindId,
    pub(crate) entity: Arc<str>,
    pub(crate) authority_identity: u64,
    pub(crate) phase:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    pub(crate) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<S, E> crate::ports::InvariantEntityIdentityInternalPort<S, E>
    for WorthQueryInvariantEntityIdentity<S, E>
{
    fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    fn kind(&self) -> KindId {
        self.kind
    }

    fn entity(&self) -> &Arc<str> {
        &self.entity
    }

    fn authority_identity(&self) -> u64 {
        self.authority_identity
    }

    fn phase(
        &self,
    ) -> Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>
    {
        self.phase
    }

    fn fragment_schema<TargetSchema>(&self) -> WorthQueryInvariantEntityIdentity<TargetSchema, ()> {
        Self::fragment_schema(self)
    }

    fn native_identity(
        &self,
    ) -> crate::domain_computation::primary_graph::invariant_projection::native_read::NativeIdentity<'_>
{
        Self::native_identity(self)
    }
}

impl crate::ports::InvariantEntityErasedInternalPort for WorthQueryInvariantEntityIdentity<(), ()> {
    fn into_typed<S, E>(self) -> WorthQueryInvariantEntityIdentity<S, E> {
        Self::into_typed(self)
    }
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
