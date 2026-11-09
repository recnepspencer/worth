//! One exhaustive visit of transaction-local key coordinates.
use super::{CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent};
use crate::{symbols::data::ClientKey, transactions::data::EntityReference};
impl MutationIntent {
    pub(crate) fn visit_client_keys_mut<E>(
        &mut self,
        mut visit: impl FnMut(&mut ClientKey) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Self::Create(CreateIntent::Entity(spec)) => visit(&mut spec.client_key)?,
            Self::Create(CreateIntent::EntityAspects(spec)) => visit(&mut spec.client_key)?,
            Self::Create(CreateIntent::Relation(spec)) => visit(&mut spec.client_key)?,
            Self::Create(CreateIntent::RelationAspects(spec)) => visit(&mut spec.client_key)?,
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                for key in &mut spec.client_keys {
                    visit(key)?;
                }
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                for key in &mut spec.client_keys {
                    visit(key)?;
                }
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                visit(&mut spec.replacement.client_key)?
            }
            Self::Entity(
                EntityMutationIntent::UpdateFields(_)
                | EntityMutationIntent::ApplyAspectPatch(_)
                | EntityMutationIntent::Delete(_)
                | EntityMutationIntent::Revalidate(_),
            )
            | Self::Relation(
                RelationMutationIntent::UpdateEndpoints(_)
                | RelationMutationIntent::ApplyAspectPatch(_)
                | RelationMutationIntent::Delete(_)
                | RelationMutationIntent::Revalidate(_),
            )
            | Self::Materialization(_) => {}
        }
        self.visit_entity_references_mut(|reference| {
            if let EntityReference::Created(created) = reference {
                visit(&mut created.client_key)?;
            }
            Ok(())
        })
    }
}
