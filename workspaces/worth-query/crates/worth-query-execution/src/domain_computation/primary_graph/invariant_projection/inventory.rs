//! Whole-kind inventories of an invariant snapshot, read on its own branch.
//!
//! One relational runtime holds every branch and shares commit versions across
//! them, so a version alone would admit a sibling's commits made at older
//! versions. Each inventory projects the snapshot onto its branch root first;
//! a snapshot that no longer projects has no readable inventory.

use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationRelationRef, ApplicationSchema,
};
use worth_relational::facade::storage::RecordLifecycleState;

use super::{
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantRelation,
};

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn entities<Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Result<
        Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        super::WorthQueryInvariantProjectionDenial,
    > {
        let Some(kind) = self.layout.entity_kind(entity.name()) else {
            return Ok(Vec::new());
        };
        let records = self.graph.with_runtime(execution, |runtime| {
            let Some(view) = runtime
                .read_truth()
                .project_snapshot(self.snapshot(), &execution)
            else {
                return Ok(Vec::new());
            };
            view.entity_records_with_projection_scope(
                execution,
                kind,
                worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                |record| Some((record.entity_id(), record.lifecycle())),
            )
            .map_err(|cause| {
                super::WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None)
            })
        })?;
        Ok(records
            .into_iter()
            .filter(|(_, lifecycle)| *lifecycle == RecordLifecycleState::Live)
            .map(|(entity_id, _)| WorthQueryInvariantEntityIdentity {
                entity_id,
                kind,
                entity: Arc::from(entity.name()),
                authority_identity: self.authority_identity,
                _marker: PhantomData,
            })
            .collect())
    }

    pub fn relations<Relation, From, To>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        super::WorthQueryInvariantProjectionDenial,
    > {
        let Some(layout) = self.layout.relation(relation.name()).cloned() else {
            return Ok(Vec::new());
        };
        let records = self.graph.with_runtime(execution, |runtime| {
            let Some(view) = runtime
                .read_truth()
                .project_snapshot(self.snapshot(), &execution)
            else {
                return Ok(Vec::new());
            };
            view.relation_records_with_projection_scope(
                execution,
                layout.kind,
                worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                |record| {
                    Some((
                        record.relation_id(),
                        record.source(),
                        record.target(),
                        record.lifecycle(),
                    ))
                },
            )
            .map_err(|cause| {
                super::WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None)
            })
        })?;
        Ok(records
            .into_iter()
            .filter(|(_, _, _, lifecycle)| *lifecycle == RecordLifecycleState::Live)
            .map(
                |(relation_id, source, target, _)| WorthQueryInvariantRelation {
                    relation_id,
                    from: WorthQueryInvariantEntityIdentity {
                        entity_id: source,
                        kind: layout.from,
                        entity: Arc::from(relation.from()),
                        authority_identity: self.authority_identity,
                        _marker: PhantomData,
                    },
                    to: WorthQueryInvariantEntityIdentity {
                        entity_id: target,
                        kind: layout.to,
                        entity: Arc::from(relation.to()),
                        authority_identity: self.authority_identity,
                        _marker: PhantomData,
                    },
                    _relation: PhantomData,
                },
            )
            .collect())
    }
}
