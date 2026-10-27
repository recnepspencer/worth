//! Read-only selectors share one exact projection snapshot and installed domain.
use super::{
    WorthQueryApplicationInvariantProjectionReader as Reader,
    WorthQueryInvariantEntityIdentity as Entity,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::{
    schema_layout::MemberDomainBinding as Domain, source_facts::SourceFactsAllocation,
};
use std::{marker::PhantomData, sync::Arc};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    runtime::PinnedDerivedRelationRead, schema::NativeMemberKey as Key, snapshots::SnapshotHandle,
};

/// A local read-only graph member; a projected case cannot become an Entity target.
pub struct WorthQueryInvariantGraphMember<S, M> {
    pub(super) key: Key,
    domain: Domain,
    entity: Option<Entity<S, M>>,
    pub(super) carrier_name: Arc<str>,
    snapshot: SnapshotHandle,
    cookie: u64,
    _source: Option<Arc<PinnedDerivedRelationRead>>,
    _allocation: Option<Arc<SourceFactsAllocation>>,
}
impl<S, M> Clone for WorthQueryInvariantGraphMember<S, M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            domain: self.domain.clone(),
            entity: self.entity.as_ref().map(clone_entity),
            carrier_name: self.carrier_name.clone(),
            snapshot: self.snapshot.clone(),
            cookie: self.cookie,
            _source: self._source.clone(),
            _allocation: self._allocation.clone(),
        }
    }
}
impl<S, M> WorthQueryInvariantGraphMember<S, M> {
    pub fn as_entity(&self) -> Option<&Entity<S, M>> {
        self.entity.as_ref()
    }
    pub fn carrier_id(&self) -> worth_relational::facade::identity::EntityId {
        self.key.carrier()
    }
    pub fn case(&self) -> Option<&str> {
        match &self.key {
            Key::Entity(_) => None,
            Key::Projected(member) => Some(&member.case),
        }
    }
}
impl<S: ApplicationSchema, M> WorthQueryInvariantGraphMember<S, M> {
    pub(super) fn check(&self, reader: &Reader<'_, S>, domain: &Domain) -> Result<(), Denial> {
        if self.cookie != reader.authority_identity
            || &self.snapshot != reader.snapshot
            || &self.domain != domain
        {
            return Err(Denial::new(
                Kind::ForeignIdentity,
                self.carrier_name.as_ref(),
            ));
        }
        Ok(())
    }
    pub(super) fn from_entity(
        reader: &Reader<'_, S>,
        entity: &Entity<S, M>,
    ) -> Result<Self, Denial> {
        if !reader.identity_is_local(entity, &entity.entity)
            || reader.layout.entity_kind(&entity.entity) != Some(entity.kind)
        {
            return Err(Denial::new(Kind::ForeignIdentity, entity.entity.as_ref()));
        }
        Ok(Self {
            key: entity.entity_id.into(),
            domain: Domain::Entity(entity.kind),
            entity: Some(clone_entity(entity)),
            carrier_name: entity.entity.clone(),
            snapshot: reader.snapshot.clone(),
            cookie: reader.authority_identity,
            _source: None,
            _allocation: None,
        })
    }
    pub(super) fn from_source(
        reader: &mut Reader<'_, S>,
        key: &Key,
        domain: &Domain,
        source: &Arc<PinnedDerivedRelationRead>,
        allocation: Option<&Arc<SourceFactsAllocation>>,
    ) -> Result<Self, Denial> {
        let kind = match domain {
            Domain::Entity(kind) => *kind,
            Domain::Projected(domain) => domain.parts().carrier,
        };
        let name = reader
            .layout
            .entity_name(kind)
            .ok_or_else(|| Denial::new(Kind::EndpointUnavailable, "derived carrier"))?;
        if let Some(allocation) = allocation {
            let bytes = name
                .len()
                .checked_add(2 * size_of::<usize>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(|| reader.walk_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow, "derived carrier"))?;
            allocation
                .claim(reader.execution, bytes)
                .map_err(|cause| reader.walk_stop(cause, "derived carrier"))?;
        }
        let name: Arc<str> = Arc::from(
            reader
                .layout
                .entity_name(kind)
                .expect("checked installed carrier"),
        );
        let entity = match (key, domain) {
            (Key::Entity(id), Domain::Entity(kind)) => Some(Entity {
                entity_id: *id,
                kind: *kind,
                entity: name.clone(),
                authority_identity: reader.authority_identity,
                _marker: PhantomData,
            }),
            (Key::Projected(_), Domain::Projected(_)) => None,
            _ => return Err(Denial::new(Kind::ForeignIdentity, name.as_ref())),
        };
        Ok(Self {
            key: key.clone(),
            domain: domain.clone(),
            entity,
            carrier_name: name,
            snapshot: reader.snapshot.clone(),
            cookie: reader.authority_identity,
            _source: Some(source.clone()),
            _allocation: allocation.cloned(),
        })
    }
}
fn clone_entity<S, M>(entity: &Entity<S, M>) -> Entity<S, M> {
    Entity {
        entity_id: entity.entity_id,
        kind: entity.kind,
        entity: entity.entity.clone(),
        authority_identity: entity.authority_identity,
        _marker: PhantomData,
    }
}
