//! Declared source-scope custody is observed by its existing Native fact reader.
use super::*;
use crate::{mvcc::RequiredOutputRolePosture as Posture, transactions::data::RecordRef};
impl BasisRead<'_, '_, '_, '_, '_, '_> {
    pub(in crate::branch::model_ownership) fn custody_neighbors(
        &mut self,
        entity: EntityId,
        outgoing: bool,
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial> {
        use super::traversal::Read;
        let Some(roster) = self.custody_roster(entity)? else {
            return Ok(Vec::new());
        };
        let mut neighbors = Vec::new();
        for origin in roster.origins() {
            for member in &origin.members {
                self.charge(1)?;
                if member.posture == Posture::Retire {
                    continue;
                }
                let neighbor = if outgoing && origin.meaning.source_scope == entity {
                    Some(member.id)
                } else if !outgoing && member.id == entity {
                    Some(origin.meaning.source_scope)
                } else {
                    None
                };
                if let Some(neighbor) = neighbor {
                    self.retain()?;
                    neighbors.push(neighbor);
                }
            }
        }
        Ok(neighbors)
    }

    fn custody_roster(
        &mut self,
        entity: EntityId,
    ) -> Result<
        Option<crate::publication::ownership::CurrentPublicationRoster>,
        RelationalModelOwnershipDenial,
    > {
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
            None => crate::publication::ownership::read_candidate(
                self.execution,
                self.root,
                &records,
                self.context,
            ),
        }
        .map_err(RelationalModelOwnershipDenial::Custody)?;
        Ok(Some(roster))
    }
    /// The roster proves ownership of IDs; the ordinary relation reader owns
    /// their current full-generation structural facts.
    pub(in crate::branch::model_ownership) fn custody_relation_ids(
        &mut self,
        entity: EntityId,
    ) -> Result<
        std::collections::BTreeSet<crate::identity::data::RelationId>,
        RelationalModelOwnershipDenial,
    > {
        let mut edges = std::collections::BTreeSet::new();
        let Some(roster) = self.custody_roster(entity)? else {
            return Ok(edges);
        };
        for origin in roster.origins() {
            for owned in &origin.relations {
                self.charge(1)?;
                if owned.source == entity && !edges.contains(&owned.id) {
                    self.retain()?;
                    edges.insert(owned.id);
                }
            }
        }
        Ok(edges)
    }
}
