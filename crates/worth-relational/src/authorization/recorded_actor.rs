//! Observe native actor coordinates on one exact snapshot.

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use crate::facade::{
    identity::{EntityId, PartitionId},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime},
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
    transactions::RelationalExecutionDenialCause,
};

/// Live proof of the actor coordinates stored in a single native record.
#[derive(Debug)]
pub struct RelationalAuthorizationRecordedActorObservation {
    snapshot: SnapshotHandle,
    actor: Option<EntityId>,
}

impl RelationalAuthorizationRecordedActorObservation {
    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub const fn actor(&self) -> Option<EntityId> {
        self.actor
    }
}

impl RelationalRuntime {
    /// Reads the partition, slot, and generation from one projected aspect.
    /// A partial or malformed tuple refuses observation.
    pub fn observe_authorization_recorded_actor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        record: EntityId,
        coordinates: [&AspectFieldLocator; 3],
    ) -> Result<
        Option<RelationalAuthorizationRecordedActorObservation>,
        RelationalExecutionDenialCause,
    > {
        let aspect = coordinates[0].aspect().aspect_key();
        let mut fields = Vec::with_capacity(coordinates.len());
        for locator in coordinates {
            let [field] = locator.field_path().fields() else {
                return Ok(None);
            };
            if locator.aspect().aspect_key() != aspect || fields.contains(field) {
                return Ok(None);
            }
            fields.push(field.clone());
        }
        let Some(view) = self.read_truth().project_snapshot(snapshot) else {
            return Ok(None);
        };
        let scope =
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                fields.iter().cloned(),
            )]);
        view.entity_record_with_projection_scope(execution, record, scope, |record| {
            if record.lifecycle() != RecordLifecycleState::Live {
                return None;
            }
            let mut components = [None; 3];
            for (component, field) in components.iter_mut().zip(&fields) {
                *component = match record.observe_aspect_field(aspect, field)?.value() {
                    Some(AspectValue::UInt64(value)) => Some(*value),
                    None => None,
                    Some(_) => return None,
                };
            }
            let actor = match components {
                [None, None, None] => None,
                [Some(partition), Some(slot), Some(generation)] => Some(EntityId::new(
                    PartitionId::new(u32::try_from(partition).ok()?),
                    slot,
                    u32::try_from(generation).ok()?,
                )),
                _ => return None,
            };
            Some(RelationalAuthorizationRecordedActorObservation {
                snapshot: snapshot.clone(),
                actor,
            })
        })
    }
}
