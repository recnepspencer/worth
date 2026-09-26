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
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>> {
        let Some(kind) = self.layout.entity_kind(entity.name()) else {
            return Vec::new();
        };
        let records = self.graph.with_runtime(|runtime| {
            runtime
                .read_truth()
                .project_snapshot(self.snapshot())
                .and_then(|view| view.bounded_entities_of_kind(kind, usize::MAX).ok())
                .map(|read| read.into_records())
                .unwrap_or_default()
        });
        records
            .into_iter()
            .filter(|record| record.lifecycle == RecordLifecycleState::Live)
            .map(|record| WorthQueryInvariantEntityIdentity {
                entity_id: record.entity_id,
                kind,
                entity: Arc::from(entity.name()),
                authority_identity: self.authority_identity,
                _marker: PhantomData,
            })
            .collect()
    }

    pub fn relations<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>> {
        let Some(layout) = self.layout.relation(relation.name()).cloned() else {
            return Vec::new();
        };
        let records = self.graph.with_runtime(|runtime| {
            runtime
                .read_truth()
                .project_snapshot(self.snapshot())
                .and_then(|view| view.bounded_relations_of_kind(layout.kind, usize::MAX).ok())
                .map(|read| read.into_records())
                .unwrap_or_default()
        });
        records
            .into_iter()
            .filter(|record| record.lifecycle == RecordLifecycleState::Live)
            .map(|record| WorthQueryInvariantRelation {
                relation_id: record.relation_id,
                from: WorthQueryInvariantEntityIdentity {
                    entity_id: record.source,
                    kind: layout.from,
                    entity: Arc::from(relation.from()),
                    authority_identity: self.authority_identity,
                    _marker: PhantomData,
                },
                to: WorthQueryInvariantEntityIdentity {
                    entity_id: record.target,
                    kind: layout.to,
                    entity: Arc::from(relation.to()),
                    authority_identity: self.authority_identity,
                    _marker: PhantomData,
                },
                _relation: PhantomData,
            })
            .collect()
    }
}
