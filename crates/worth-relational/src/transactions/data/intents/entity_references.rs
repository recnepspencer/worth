//! One exhaustive traversal of the symbolic entity endpoints in an intent.
use super::{CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent};
use crate::transactions::data::EntityReference;

impl MutationIntent {
    pub(crate) fn visit_entity_references_mut<E>(
        &mut self,
        mut visit: impl FnMut(&mut EntityReference) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Self::Create(CreateIntent::Relation(spec)) => {
                visit(&mut spec.source)?;
                visit(&mut spec.target)?;
            }
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                visit(&mut spec.source)?;
                visit(&mut spec.target)?;
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                for (source, target) in &mut spec.endpoints {
                    visit(source)?;
                    visit(target)?;
                }
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                visit(&mut spec.source)?;
                visit(&mut spec.target)?;
            }
            Self::Create(
                CreateIntent::Entity(_)
                | CreateIntent::EntityAspects(_)
                | CreateIntent::BulkEntities(_),
            )
            | Self::Entity(
                EntityMutationIntent::UpdateFields(_)
                | EntityMutationIntent::ApplyAspectPatch(_)
                | EntityMutationIntent::Replace(_)
                | EntityMutationIntent::Delete(_)
                | EntityMutationIntent::Revalidate(_),
            )
            | Self::Relation(
                RelationMutationIntent::ApplyAspectPatch(_)
                | RelationMutationIntent::Delete(_)
                | RelationMutationIntent::Revalidate(_),
            )
            | Self::Materialization(_) => {}
        }
        Ok(())
    }
}
