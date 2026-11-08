//! Authored DATA cannot select private Query records or maintenance mutations.
use super::author_source::{
    WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
};
use worth_foundational::facade::HomeProgramTransitionStepRole;
use worth_relational::facade::{
    identity::{EntityId, KindId, RelationId},
    runtime::RelationalBorrowedRecordReadDenial,
    transactions::*,
};

#[derive(Debug)]
pub enum WorthQueryHomeProgramTransitionBodyDenial {
    Request(worth_execution::WorkCeilingDenial),
    Read(RelationalBorrowedRecordReadDenial),
    EntityKindNotAuthored(KindId),
    RelationKindNotAuthored(KindId),
    EntityNotAuthored(EntityId),
    RelationNotAuthored(RelationId),
    MaterializationNotAuthored,
}
impl WorthQueryHomeProgramTransitionReadSource<'_, '_> {
    /// Check the entire user body roster before the framework appends its one
    /// activation update and before Home reserves or publishes the intent.
    pub fn admit_authored_body(
        &self,
        target: &WorthQueryHomeProgramTransitionTarget<'_>,
        role: HomeProgramTransitionStepRole,
        batches: &[WorkerIntentBatch],
    ) -> Result<(), WorthQueryHomeProgramTransitionBodyDenial> {
        use WorthQueryHomeProgramTransitionBodyDenial as Denial;
        let allowed = match role {
            HomeProgramTransitionStepRole::BeforeTarget => self.layout,
            HomeProgramTransitionStepRole::Target | HomeProgramTransitionStepRole::AfterTarget => {
                target.layout
            }
        };
        let entity_kind = |kind| {
            if allowed.is_application_entity_kind(kind) {
                Ok(())
            } else {
                Err(Denial::EntityKindNotAuthored(kind))
            }
        };
        let relation_kind = |kind| {
            if allowed.is_application_relation_kind(kind) {
                Ok(())
            } else {
                Err(Denial::RelationKindNotAuthored(kind))
            }
        };
        let entity = |id| {
            let kind = self
                .view
                .read_exact_entity_state(&self.request(), id, |metadata, _| metadata.kind_id)
                .map_err(Denial::Read)?;
            match kind {
                Some(kind) if allowed.is_application_entity_kind(kind) => Ok(()),
                Some(_) | None => Err(Denial::EntityNotAuthored(id)),
            }
        };
        let relation = |id| {
            let metadata = self
                .view
                .read_exact_relation_metadata(&self.request(), id)
                .map_err(Denial::Read)?;
            match metadata {
                Some(metadata) if allowed.is_application_relation_kind(metadata.kind_id) => Ok(()),
                Some(_) | None => Err(Denial::RelationNotAuthored(id)),
            }
        };
        let endpoint = |reference: &EntityReference| match reference {
            EntityReference::Existing(id) => entity(*id),
            EntityReference::Created(_) => Ok(()), // Every create is separately checked; Native owns reference validity.
        };
        for batch in batches {
            self.request().consult().map_err(Denial::Request)?;
            for mutation in &batch.intents {
                self.request().consult().map_err(Denial::Request)?;
                match mutation {
                    MutationIntent::Create(create) => match create {
                        CreateIntent::Entity(spec) => entity_kind(spec.kind_id)?,
                        CreateIntent::EntityAspects(spec) => entity_kind(spec.kind_id)?,
                        CreateIntent::BulkEntities(spec) => entity_kind(spec.kind_id)?,
                        CreateIntent::Relation(spec) => {
                            relation_kind(spec.kind_id)?;
                            endpoint(&spec.source)?;
                            endpoint(&spec.target)?;
                        }
                        CreateIntent::RelationAspects(spec) => {
                            relation_kind(spec.kind_id)?;
                            endpoint(&spec.source)?;
                            endpoint(&spec.target)?;
                        }
                        CreateIntent::BulkRelations(spec) => {
                            relation_kind(spec.kind_id)?;
                            for (source, target) in &spec.endpoints {
                                self.request().consult().map_err(Denial::Request)?;
                                endpoint(source)?;
                                endpoint(target)?;
                            }
                        }
                    },
                    MutationIntent::Entity(change) => match change {
                        EntityMutationIntent::UpdateFields(change) => entity(change.entity_id)?,
                        EntityMutationIntent::ApplyAspectPatch(change) => entity(change.entity_id)?,
                        EntityMutationIntent::Replace(change) => {
                            entity(change.entity_id)?;
                            entity_kind(change.replacement.kind_id)?;
                        }
                        EntityMutationIntent::Delete(change) => entity(change.entity_id)?,
                        EntityMutationIntent::Revalidate(change) => entity(change.entity_id)?,
                    },
                    MutationIntent::Relation(change) => match change {
                        RelationMutationIntent::UpdateEndpoints(change) => {
                            relation(change.relation_id)?;
                            relation_kind(change.kind_id)?;
                            endpoint(&change.source)?;
                            endpoint(&change.target)?;
                        }
                        RelationMutationIntent::ApplyAspectPatch(change) => {
                            relation(change.relation_id)?
                        }
                        RelationMutationIntent::Delete(change) => relation(change.relation_id)?,
                        RelationMutationIntent::Revalidate(change) => relation(change.relation_id)?,
                    },
                    MutationIntent::Materialization(_) => {
                        return Err(Denial::MaterializationNotAuthored)
                    }
                }
            }
        }
        Ok(())
    }
}
