//! Observes one decision fact at a snapshot: the observation seal makes, and
//! the one a partitioned computation makes of the facts it retained.

use worth_relational::facade::runtime::{ProjectionAspectScope, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::storage::RecordLifecycleState;

use super::super::fact::{
    observe_adjacency, WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact,
};
use super::denial;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

/// The fact `key` names as `snapshot` holds it, or the denial seal gives when
/// it cannot be observed there.
pub(in crate::domain_computation::primary_graph) fn observe_fact(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryPrimaryGraphLayout,
    key: &WorthQueryApplicationFactKey,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    match key {
        WorthQueryApplicationFactKey::Entity { entity, entity_id } => {
            let kind = layout.entity_kind(entity).ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    entity,
                )
            })?;
            let exists = runtime
                .read_truth()
                .project_snapshot(snapshot)
                .and_then(|view| {
                    view.entity_record_with_projection_scope(
                        *entity_id,
                        ProjectionAspectScope::empty(),
                        |record| {
                            Some(
                                record.kind_id() == kind
                                    && record.lifecycle() == RecordLifecycleState::Live,
                            )
                        },
                    )
                })
                .unwrap_or(false);
            if !exists {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    entity,
                ));
            }
            Ok(WorthQueryApplicationObservedFact::Entity {
                entity_id: *entity_id,
                kind,
            })
        }
        WorthQueryApplicationFactKey::Field {
            entity,
            entity_id,
            locator,
        } => {
            let kind = layout.entity_kind(entity).ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    entity,
                )
            })?;
            let observation = super::super::observation::observe_field(
                runtime, snapshot, *entity_id, kind, locator,
            )
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    entity,
                )
            })?;
            let fact = match observation {
                super::super::observation::WorthQueryApplicationFieldObservation::Present(
                    value,
                ) => WorthQueryApplicationObservedFact::Field {
                    entity_id: *entity_id,
                    kind,
                    locator: locator.clone(),
                    value,
                },
                super::super::observation::WorthQueryApplicationFieldObservation::Absent
                    if layout.field_is_optional(entity, locator) =>
                {
                    WorthQueryApplicationObservedFact::AbsentField {
                        entity_id: *entity_id,
                        kind,
                        locator: locator.clone(),
                    }
                }
                super::super::observation::WorthQueryApplicationFieldObservation::Absent => {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                        entity,
                    ));
                }
            };
            Ok(fact)
        }
        WorthQueryApplicationFactKey::Relation { relation, from, to } => {
            let layout = layout.relation(relation).ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    relation,
                )
            })?;
            let matching_relations = super::super::observation::exact_relations(
                runtime,
                snapshot,
                layout.kind,
                *from,
                *to,
            )?;
            Ok(WorthQueryApplicationObservedFact::Relation {
                relation_kind: layout.kind,
                from: *from,
                to: *to,
                matching_relations,
            })
        }
        WorthQueryApplicationFactKey::Adjacency {
            relation,
            anchor,
            direction,
            maximum_work_units,
        } => {
            let layout = layout.relation(relation).ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    relation,
                )
            })?;
            let relations = observe_adjacency(
                runtime,
                snapshot,
                layout.kind,
                *anchor,
                *direction,
                *maximum_work_units,
            );
            let relations = relations.ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                    relation,
                )
            })?;
            Ok(WorthQueryApplicationObservedFact::Adjacency {
                relation_kind: layout.kind,
                anchor: *anchor,
                direction: *direction,
                maximum_work_units: *maximum_work_units,
                relations,
            })
        }
    }
}
