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
        if self
            .root
            .schema_authority()
            .registry()
            .current_publication_storage
            .is_none()
        {
            return Ok(Vec::new());
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
}
