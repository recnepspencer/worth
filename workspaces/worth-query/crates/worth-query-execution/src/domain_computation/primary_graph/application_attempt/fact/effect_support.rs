//! Compare the same decision coordinates after the writer's own effect and at group completion.
use super::super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use super::{image::FactReadImage, WorthQueryApplicationObservedFact as Fact};
use worth_relational::facade::{
    runtime::{
        RelationalAdjacencySupportDenial as EdgeStop, RelationalAppliedCandidateReadIdentity,
        RelationalFieldSupportComparisonDenial as FieldStop, RelationalPinnedRead,
        RelationalRuntime,
    },
    snapshots::SnapshotHandle,
};
type Candidate = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
impl Fact {
    pub(in crate::domain_computation::primary_graph) fn support_equal_after_own_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        after_own: &Candidate,
        complete: &Candidate,
    ) -> Result<bool, Denial> {
        execution.consult().map_err(|cause| {
            Denial::relational_read_stopped(cause.into(), "coupled decision support")
        })?;
        let left = FactReadImage {
            runtime,
            before,
            applied: Some(after_own),
        };
        let right = FactReadImage {
            runtime,
            before,
            applied: Some(complete),
        };
        let a = after_own.project_candidate();
        let b = complete.project_candidate();
        let physical = |cause| Denial::relational_read_stopped(cause, "coupled decision support");
        let retired = |entity| -> Result<Option<bool>, Denial> {
            let l = a
                .read_entity_retirement(execution, entity)
                .map_err(physical)?;
            let r = b
                .read_entity_retirement(execution, entity)
                .map_err(physical)?;
            Ok(match (l, r) {
                (None, None) => None,
                (Some(l), Some(r)) => Some(
                    l.kind_id() == r.kind_id()
                        && l.created_at_version() == r.created_at_version()
                        && l.deleted_at_version() == r.deleted_at_version(),
                ),
                _ => Some(false),
            })
        };
        match self {
            Self::Entity { entity_id, .. }
            | Self::SourceEntity { entity_id }
            | Self::RetiredOutputEntity { entity_id, .. } => {
                if let Some(equal) = retired(*entity_id)? {
                    return Ok(equal);
                }
                Ok(left.live_kind(execution, *entity_id).map_err(physical)?
                    == right.live_kind(execution, *entity_id).map_err(physical)?)
            }
            Self::Field {
                entity_id, locator, ..
            }
            | Self::AbsentField {
                entity_id, locator, ..
            }
            | Self::SourceFieldRevision {
                entity_id, locator, ..
            } => {
                if let Some(equal) = retired(*entity_id)? {
                    return Ok(equal);
                }
                let locus = worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    locator.aspect().aspect_key().clone(),
                    locator.field_path().clone(),
                );
                a.compare_entity_field_support(execution, &b, *entity_id, &locus)
                    .map_err(field_stop)
            }
            Self::OptionalAspectAbsent {
                entity_id, locator, ..
            } => {
                if let Some(equal) = retired(*entity_id)? {
                    return Ok(equal);
                }
                a.compare_entity_aspect_support(
                    execution,
                    &b,
                    *entity_id,
                    locator.aspect().aspect_key(),
                )
                .map_err(field_stop)
            }
            Self::SourceAspectRevision {
                entity_id, aspect, ..
            }
            | Self::AspectPresence {
                entity_id, aspect, ..
            } => {
                if let Some(equal) = retired(*entity_id)? {
                    return Ok(equal);
                }
                a.compare_entity_aspect_support(execution, &b, *entity_id, aspect)
                    .map_err(field_stop)
            }
            Self::Relation {
                relation_kind,
                from,
                to,
                ..
            } => {
                if let Some(equal) = retired(*from)? {
                    return Ok(equal);
                }
                Ok(left.relations(execution, *relation_kind, *from, *to)?
                    == right.relations(execution, *relation_kind, *from, *to)?)
            }
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                ..
            } => {
                if let Some(equal) = retired(*anchor)? {
                    return Ok(equal);
                }
                Ok(left.adjacency(
                    execution,
                    *relation_kind,
                    *anchor,
                    *direction,
                    *maximum_work_units,
                )? == right.adjacency(
                    execution,
                    *relation_kind,
                    *anchor,
                    *direction,
                    *maximum_work_units,
                )?)
            }
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                ..
            } => {
                if let Some(equal) = retired(*anchor)? {
                    return Ok(equal);
                }
                a.compare_adjacency_support(execution, &b, *anchor, *relation_kind, *direction)
                    .map_err(|cause| match cause {
                        EdgeStop::Execution(cause) => physical(cause),
                        EdgeStop::Allocation(cause) => {
                            Denial::allocation_denied("coupled adjacency support", cause)
                        }
                        _ => unavailable(),
                    })
            }
            Self::IndexedEntitySelection { .. } => {
                super::indexed_entity_selection::applied::compare(
                    self,
                    execution,
                    runtime,
                    before,
                    Some(after_own),
                    complete,
                )
            }
            Self::StructuralWalk { observation } => observation
                .support_equal_between(execution, after_own, complete)
                .map_err(source_stop),
            Self::Aggregate { observation } => observation
                .support_equal_between(execution, after_own, complete)
                .map_err(source_stop),
            Self::ComponentSelection { observation } => {
                observation.support_equal_between(execution, runtime, before, after_own, complete)
            }
            // Control and external publication observations retain their ordinary
            // owners. A moved unsupported observation refuses, never grants support.
            _ => Ok(
                self.remains_equal_in_image(runtime, execution, before, Some(after_own))?
                    && self.remains_equal_in_image(runtime, execution, before, Some(complete))?,
            ),
        }
    }
}
fn unavailable() -> Denial {
    Denial::new(Kind::MissingAuthoritativeFact, "coupled decision support")
}
fn field_stop(cause: FieldStop) -> Denial {
    match cause {
        FieldStop::Execution(cause) => {
            Denial::relational_read_stopped(cause, "coupled field support")
        }
        _ => Denial::new(
            Kind::MissingAuthoritativeFact,
            format!("coupled field support: {cause:?}"),
        ),
    }
}
pub(in crate::domain_computation::primary_graph) fn source_stop(
    cause: super::WorthQuerySourceCurrentnessFailure,
) -> Denial {
    use super::WorthQuerySourceCurrentnessFailure as Stop;
    match cause {
        Stop::Execution(cause) => {
            Denial::relational_read_stopped(cause, "coupled structural support")
        }
        Stop::WorkBudgetExceeded => Denial::new(
            Kind::DecisionFactBudgetExceeded,
            "coupled structural support",
        ),
        Stop::OutputObservation(cause) => Denial::output_observation(cause),
        Stop::ManagedIndex(cause) => Denial::managed_index(cause, "coupled structural support"),
        Stop::Unavailable => unavailable(),
    }
}
