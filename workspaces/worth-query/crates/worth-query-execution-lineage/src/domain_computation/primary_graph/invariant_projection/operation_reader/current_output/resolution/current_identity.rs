//! Verify the live entity kind and authorize the selected output identity.

use super::*;
use std::marker::PhantomData;
use crate::domain_computation::primary_graph::output_lineage::family_selection::{
    live_selection_identity, LiveSelectionRefusal,
};

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn live_current_identity<Entity>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.live_identity_in::<Entity>(execution, role, entity_id, false)
    }

    /// As `live_current_identity`, but a record a successor round's own
    /// chain wrote is admitted as it was at the chain head (§1.5), where
    /// every other read of that record is served.
    pub(crate) fn live_own_view_identity<Entity>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.live_identity_in::<Entity>(execution, role, entity_id, true)
    }

    fn live_identity_in<Entity>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        role: &str,
        entity_id: worth_relational::facade::identity::EntityId,
        own_view: bool,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let native = &self.reader.native;
        let view = if own_view {
            native.projection_of(self.reader.snapshot, entity_id)
        } else {
            native.projection(self.reader.snapshot)
        };
        let live = live_selection_identity(
            view.as_ref(),
            execution,
            entity_id,
            self.reader.layout,
            Entity::IDENTIFIER,
        )
        .map_err(|cause| {
            self.reader.native_read_stop.get_or_insert(cause);
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                role,
            )
        })?;
        let (kind, entity) = live.map_err(|refusal| {
            WorthQueryCurrentOutputDenial::new(
                match refusal {
                    LiveSelectionRefusal::OutputUnavailable => {
                        WorthQueryCurrentOutputDenialKind::OutputUnavailable
                    }
                    LiveSelectionRefusal::EntityMismatch => {
                        WorthQueryCurrentOutputDenialKind::EntityMismatch
                    }
                },
                role,
            )
        })?;
        self.reader.realized_scope.record(entity_id);
        let identity = WorthQueryInvariantEntityIdentity {
            entity_id,
            kind,
            entity: std::sync::Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            phase: self
                .reader
                .native
                .candidate_identity()
                .map(|id| id.qualification()),
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
