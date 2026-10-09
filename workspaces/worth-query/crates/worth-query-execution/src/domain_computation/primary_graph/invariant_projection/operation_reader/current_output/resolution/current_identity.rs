//! Verify the live entity kind and authorize the selected output identity.

use super::*;
use std::marker::PhantomData;
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub(super) fn live_current_identity<Entity>(
        &mut self,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let projected = self
            .reader
            .runtime
            .read_truth()
            .project_snapshot(self.reader.snapshot)
            .and_then(|view| {
                view.entity_record_with_projection_scope(
                    entity_id,
                    ProjectionAspectScope::empty(),
                    |record| Some((record.kind_id(), record.lifecycle())),
                )
            })
            .filter(|(_, lifecycle)| *lifecycle == RecordLifecycleState::Live)
            .ok_or_else(|| {
                WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                    role,
                )
            })?;
        let entity = self.reader.layout.entity_name(projected.0).ok_or_else(|| {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::EntityMismatch,
                role,
            )
        })?;
        if entity != Entity::IDENTIFIER {
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::EntityMismatch,
                role,
            ));
        }
        self.reader.realized_scope.record(entity_id);
        let identity = WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: projected.0,
            entity: std::sync::Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            _marker: PhantomData,
        };
        self.require_decision_entity(
            &identity,
            ApplicationEntityRef::from_schema_identifier(Entity::IDENTIFIER),
        )
        .map_err(|denial| decision_plan_denial(denial.kind(), role))?;
        Ok(identity)
    }
}
