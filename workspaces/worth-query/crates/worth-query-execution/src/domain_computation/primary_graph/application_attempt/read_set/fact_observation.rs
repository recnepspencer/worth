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

/// The most work observing `key` can take, in the relational work units its
/// read is bounded by, or `None` when nothing bounds it. An entity or a field
/// is one record read, and an adjacency reads at most its declared units and
/// its anchor's list. A relation scans its source's whole outgoing
/// neighborhood, which no key declares a bound for.
pub(super) fn observation_work_bound(key: &WorthQueryApplicationFactKey) -> Option<u64> {
    match key {
        WorthQueryApplicationFactKey::SourcePublication(_)
        | WorthQueryApplicationFactKey::StructuralWalk(_)
        | WorthQueryApplicationFactKey::Aggregate(_) => None,
        WorthQueryApplicationFactKey::Entity { .. }
        | WorthQueryApplicationFactKey::Field { .. }
        | WorthQueryApplicationFactKey::AspectPresence { .. } => Some(1),
        WorthQueryApplicationFactKey::Relation { .. } => None,
        WorthQueryApplicationFactKey::Adjacency {
            maximum_work_units, ..
        } => u64::try_from(*maximum_work_units).ok()?.checked_add(1),
    }
}

/// The fact `key` names as `snapshot` holds it, or the denial seal gives when
/// it cannot be observed there.
pub(in crate::domain_computation::primary_graph) fn observe_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryPrimaryGraphLayout,
    key: &WorthQueryApplicationFactKey,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    observe_fact_inner(execution, runtime, snapshot, layout, key, false)
}

/// Retained comparisons have already been bounded by the computation's
/// declaration. They do not add work a fresh call would not perform.
pub(in crate::domain_computation::primary_graph) fn observe_admitted_fact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryPrimaryGraphLayout,
    key: &WorthQueryApplicationFactKey,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    super::super::super::fact_verification_observation::retained_native();
    observe_fact_inner(execution, runtime, snapshot, layout, key, true)
}

fn observe_fact_inner(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryPrimaryGraphLayout,
    key: &WorthQueryApplicationFactKey,
    admitted: bool,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    match key {
        WorthQueryApplicationFactKey::SourcePublication(_)
        | WorthQueryApplicationFactKey::StructuralWalk(_)
        | WorthQueryApplicationFactKey::Aggregate(_) => Err(denial(
            WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
            "fresh source-query read required",
        )),
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
                .map(|view| {
                    let project =
                        |record: worth_relational::facade::runtime::EntityProjectionRecord<'_>| {
                            Some(
                                record.kind_id() == kind
                                    && record.lifecycle() == RecordLifecycleState::Live,
                            )
                        };
                    let observation = if admitted {
                        view.entity_record_with_projection_scope_admitted(
                            execution,
                            *entity_id,
                            ProjectionAspectScope::empty(),
                            |_| Ok::<_, std::convert::Infallible>(()),
                            project,
                        )
                        .map(|result| result.unwrap_or_else(|never| match never {}))
                    } else {
                        view.entity_record_with_projection_scope(
                            execution,
                            *entity_id,
                            ProjectionAspectScope::empty(),
                            project,
                        )
                    };
                    observation.map_err(|cause| {
                        WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, entity)
                    })
                })
                .transpose()?
                .flatten()
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
        WorthQueryApplicationFactKey::AspectPresence {
            entity,
            entity_id,
            locator,
        } => {
            let aspect = locator.aspect().aspect_key();
            let kind = layout.entity_kind(entity).ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    entity,
                )
            })?;
            let present = super::super::observation::observe_aspect_presence(
                runtime, execution, snapshot, *entity_id, kind, aspect,
            )
            .map_err(|cause| {
                WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, entity)
            })?
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    entity,
                )
            })?;
            Ok(WorthQueryApplicationObservedFact::AspectPresence {
                entity_id: *entity_id,
                kind,
                aspect: aspect.clone(),
                present,
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
            let observation = if admitted {
                super::super::observation::observe_field_observation_borrowed_admitted(
                    runtime,
                    execution,
                    snapshot,
                    *entity_id,
                    kind,
                    locator,
                    |raw| {
                        super::super::observation::WorthQueryApplicationFieldObservation::from(raw)
                    },
                )
            } else {
                super::super::observation::observe_field(
                    runtime, execution, snapshot, *entity_id, kind, locator,
                )
            }?
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
                super::super::observation::WorthQueryApplicationFieldObservation::AspectAbsent
                    if layout.aspect_is_optional(entity, locator) => {
                        WorthQueryApplicationObservedFact::OptionalAspectAbsent {
                            entity_id: *entity_id, kind, locator: locator.clone(),
                        }
                    }
                super::super::observation::WorthQueryApplicationFieldObservation::AspectAbsent
                | super::super::observation::WorthQueryApplicationFieldObservation::UnsupportedAspectShape
                | super::super::observation::WorthQueryApplicationFieldObservation::Absent => {
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
                execution,
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
                execution,
                snapshot,
                layout.kind,
                *anchor,
                *direction,
                *maximum_work_units,
            )?;
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
