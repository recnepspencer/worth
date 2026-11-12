//! Declared source-scope custody is observed by its existing Native fact reader.
use super::*;
use crate::identity::data::RelationId;
use crate::publication::ownership::CurrentPublicationRoster;
use crate::{mvcc::RequiredOutputRolePosture as Posture, transactions::data::RecordRef};
use std::rc::Rc;

/// What the ownership walk consumes of one entity's authenticated roster:
/// every member with its origin's source scope and whether it is retired, and
/// every owned relation with its source.
pub(super) struct CustodyIncidence {
    members: Vec<(EntityId, EntityId, bool)>,
    relations: Vec<(EntityId, RelationId)>,
}

fn members(
    roster: &CurrentPublicationRoster,
) -> impl Iterator<Item = (EntityId, EntityId, bool)> + '_ {
    roster.origins().flat_map(|origin| {
        origin.members.iter().map(move |member| {
            (
                origin.meaning.source_scope,
                member.id,
                member.posture == Posture::Retire,
            )
        })
    })
}

fn relations(
    roster: &CurrentPublicationRoster,
) -> impl Iterator<Item = (EntityId, RelationId)> + '_ {
    roster.origins().flat_map(|origin| {
        origin
            .relations
            .iter()
            .map(|owned| (owned.source, owned.id))
    })
}

impl BasisRead<'_, '_, '_, '_, '_, '_> {
    pub(in crate::branch::model_ownership) fn custody_neighbors(
        &mut self,
        entity: EntityId,
        outgoing: bool,
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial> {
        if self.custody.is_none() {
            let Some(roster) = self.custody_roster(entity)? else {
                return Ok(Vec::new());
            };
            return self.neighbors_among(entity, outgoing, members(&roster));
        }
        let Some(known) = self.known_custody(entity)? else {
            return Ok(Vec::new());
        };
        self.neighbors_among(entity, outgoing, known.members.iter().copied())
    }

    fn neighbors_among(
        &mut self,
        entity: EntityId,
        outgoing: bool,
        members: impl Iterator<Item = (EntityId, EntityId, bool)>,
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial> {
        use super::traversal::Read;
        let mut neighbors = Vec::new();
        for (scope, member, retired) in members {
            self.charge(1)?;
            if retired {
                continue;
            }
            let neighbor = if outgoing && scope == entity {
                Some(member)
            } else if !outgoing && member == entity {
                Some(scope)
            } else {
                None
            };
            if let Some(neighbor) = neighbor {
                self.retain()?;
                neighbors.push(neighbor);
            }
        }
        Ok(neighbors)
    }

    /// A candidate observation holds one immutable root for its whole life, so
    /// an entity's custody is read and authenticated once and then remembered
    /// under the exact record of that read. An admitted-basis observation never
    /// comes here: each of its reads is an observed contact.
    fn known_custody(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<Rc<CustodyIncidence>>, RelationalModelOwnershipDenial> {
        if let Some(known) = self.custody.as_ref().and_then(|known| known.get(&entity)) {
            return Ok(Some(Rc::clone(known)));
        }
        let Some(roster) = self.custody_roster(entity)? else {
            return Ok(None);
        };
        let (member_count, relation_count) = (members(&roster).count(), relations(&roster).count());
        self.retain_bytes(
            (512 + member_count * size_of::<(EntityId, EntityId, bool)>()
                + relation_count * size_of::<(EntityId, RelationId)>()) as u64,
        )?;
        let mut incidence = CustodyIncidence {
            members: Vec::with_capacity(member_count),
            relations: Vec::with_capacity(relation_count),
        };
        incidence.members.extend(members(&roster));
        incidence.relations.extend(relations(&roster));
        let incidence = Rc::new(incidence);
        if let Some(known) = self.custody.as_mut() {
            known.insert(entity, Rc::clone(&incidence));
        }
        Ok(Some(incidence))
    }

    fn custody_roster(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<CurrentPublicationRoster>, RelationalModelOwnershipDenial> {
        if self
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .is_none()
        {
            return Ok(None);
        }
        let records = [RecordRef::Entity(entity)];
        let roster = match self.basis {
            Some(basis) => {
                crate::publication::ownership::read(self.execution, basis, &records, self.context)
            }
            None => {
                crate::branch::model_ownership::custody_observation::record(entity);
                crate::publication::ownership::read_candidate(
                    self.execution,
                    self.root,
                    &records,
                    self.context,
                )
            }
        }
        .map_err(RelationalModelOwnershipDenial::Custody)?;
        Ok(Some(roster))
    }
    /// The roster proves ownership of IDs; the ordinary relation reader owns
    /// their current full-generation structural facts.
    pub(in crate::branch::model_ownership) fn custody_relation_ids(
        &mut self,
        entity: EntityId,
    ) -> Result<std::collections::BTreeSet<RelationId>, RelationalModelOwnershipDenial> {
        if self.custody.is_none() {
            let Some(roster) = self.custody_roster(entity)? else {
                return Ok(std::collections::BTreeSet::new());
            };
            return self.relations_among(entity, relations(&roster));
        }
        let Some(known) = self.known_custody(entity)? else {
            return Ok(std::collections::BTreeSet::new());
        };
        self.relations_among(entity, known.relations.iter().copied())
    }

    fn relations_among(
        &mut self,
        entity: EntityId,
        relations: impl Iterator<Item = (EntityId, RelationId)>,
    ) -> Result<std::collections::BTreeSet<RelationId>, RelationalModelOwnershipDenial> {
        use super::traversal::Read;
        let mut edges = std::collections::BTreeSet::new();
        for (source, id) in relations {
            self.charge(1)?;
            if source == entity && !edges.contains(&id) {
                self.retain()?;
                edges.insert(id);
            }
        }
        Ok(edges)
    }
}
