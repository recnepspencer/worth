use super::super::WorthQueryApplicationAttemptDenial as SelfDenial;
use super::{
    indexed_entity_selection, workflow_definition_predecessor, workflow_history_basis,
    workflow_instance_capacity, WorthQueryApplicationObservedFact,
};

impl WorthQueryApplicationObservedFact {
    pub(crate) fn remains_equal_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, super::super::WorthQueryApplicationAttemptDenial> {
        self.remains_equal_in_image(runtime, execution, snapshot, None)
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal_in_image(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        applied: Option<
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        >,
    ) -> Result<bool, super::super::WorthQueryApplicationAttemptDenial> {
        let image = super::image::FactReadImage {
            runtime,
            before: snapshot,
            applied,
        };
        super::super::super::fact_verification_observation::value_comparison();
        let check_request = || {
            execution.consult().map_err(|stop| {
                SelfDenial::query_read_denial(
                    worth_relational::facade::runtime::QueryReadExecutionStop::Request(stop),
                    "decision fact comparison",
                )
            })
        };
        check_request()?;
        let equal = match self {
            Self::SourceReadAuthorization { observation } => { observation.validate_current(runtime, execution, snapshot).map_err(SelfDenial::output_observation)?; true },
            Self::ManagedIndexSource { observation } => observation.remains_equal(runtime, execution, snapshot).map_err(|cause| match cause {
                super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "managed Source comparison"),
                super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause, "managed Source comparison"),
                _ => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, "managed Source comparison"),
            })?,
            Self::ComponentSelection { observation } => match applied {
                None => observation.remains_equal(runtime,execution,snapshot,None),
                Some(pin) => crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification::open_applied(runtime, snapshot, pin).check_component(observation, execution, None),
            }
                .map_err(|cause| match cause {
                    super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                    super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "decision source consistency"),
                    super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause,"component selection comparison"),
                    super::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,"component selection comparison"),
                    super::WorthQuerySourceCurrentnessFailure::Unavailable => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,"component selection comparison"),
                })?,
            Self::Aggregate { observation } => match applied {
                None => observation.remains_equal(runtime,execution,snapshot,None),
                Some(pin) => crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification::open_applied(runtime, snapshot, pin).check_aggregate(observation, execution, None),
            }
                .map_err(|cause| match cause {
                    super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                    super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "decision source consistency"),
                    super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause,"aggregate comparison"),
                    super::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,"aggregate comparison"),
                    super::WorthQuerySourceCurrentnessFailure::Unavailable => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,"aggregate comparison"),
                })?,
            Self::DerivedSource { observation } => match applied {
                None => observation.remains_equal(runtime,execution,snapshot),
                Some(pin) => crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification::open_applied(runtime, snapshot, pin).check_derived(observation, execution, None),
            }
                .map_err(|cause| match cause {
                    super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                    super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "decision source consistency"),
                    super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause,"derived source comparison"),
                    super::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,"derived source comparison"),
                    super::WorthQuerySourceCurrentnessFailure::Unavailable => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,"derived source comparison"),
                })?,
            Self::StructuralWalk { observation } => match applied {
                None => observation.remains_equal(runtime,execution,snapshot),
                Some(pin) => crate::domain_computation::primary_graph::source_read_verification::SourceReadVerification::open_applied(runtime, snapshot, pin).check_walk(observation, execution, None),
            }
                .map_err(|cause| match cause {
                    super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                    super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "decision source consistency"),
                    super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause,"structural walk comparison"),
                    super::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,"structural walk comparison"),
                    super::WorthQuerySourceCurrentnessFailure::Unavailable => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,"structural walk comparison"),
                })?,
            Self::SourcePublication { observation } => observation.remains_equal(runtime, execution, snapshot)
                .map_err(|cause| match cause {
                    super::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => SelfDenial::output_observation(stop),
                    super::WorthQuerySourceCurrentnessFailure::ManagedIndex(cause) => SelfDenial::managed_index(cause, "decision source consistency"),
                    super::WorthQuerySourceCurrentnessFailure::Execution(cause) => SelfDenial::relational_read_stopped(cause, "source publication comparison"),
                    super::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded, "source publication comparison"),
                    super::WorthQuerySourceCurrentnessFailure::Unavailable => SelfDenial::new(super::super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, "source publication comparison"),
                })?,
            Self::RetiredOutputEntity { entity_id, kind, created_at, deleted_at, .. } => super::retirement_equality::remains_equal(image.view().as_ref(), execution, *entity_id, *kind, *created_at, *deleted_at).map_err(|cause| SelfDenial::relational_read_stopped(cause, "retirement comparison"))?,
            Self::SourceEntity { entity_id } => match image.view() {
                Some(view) => {
                    view.entity_record_with_projection_scope(execution,
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(
                                record.lifecycle()
                                    == worth_relational::facade::storage::RecordLifecycleState::Live,
                            )
                        },
                    ).map_err(|cause| SelfDenial::relational_read_stopped(cause, "source entity comparison"))? == Some(true)
                },
                None => false,
            },
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => image.aspect_revision(execution, *entity_id, aspect)?
                == Some(*native_revision),
            Self::SourceFieldRevision { entity_id, locator, native_revision } =>
                if let Some(expected) = native_revision {
                    match image.view() {
                        Some(view) => view.entity_field_revision(execution, *entity_id, locator)
                            .map_err(|cause| SelfDenial::relational_read_stopped(cause, "source field comparison"))? == Some(*expected),
                        None => false,
                    }
                } else { false },
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                ..
            } => match image.view() {
                Some(view) => {
                    match view.bounded_adjacency_structural_revision(execution,
                        *anchor,
                        *relation_kind,
                        *direction,
                        *comparison_work_limit,
                    ) {
                        Ok(current) => current.revision() == *native_revision,
                        Err(worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::Execution(cause)) =>
                            return Err(SelfDenial::relational_read_stopped(cause, "source adjacency comparison")),
                        // A missing basis or insufficient declared comparison
                        // bound cannot prove equality. Request failures stay typed.
                        Err(
                            worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::AnchorUnavailable
                            | worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial::BasisUnavailable,
                        ) => false,
                    }
                },
                None => false,
            },
            Self::Entity {
                entity_id, kind, ..
            } => image.live_kind(execution, *entity_id)
                .map_err(|cause| SelfDenial::relational_read_stopped(cause, "source entity comparison"))?
                == Some(Some(*kind)),
            Self::Field {
                entity_id,
                kind,
                locator,
                value,
                ..
            } => image.field(execution, *entity_id, *kind, locator)?
                .is_some_and(|current| matches!(current, super::super::observation::WorthQueryApplicationFieldObservation::Present(current) if current == *value)),
            Self::AspectPresence { entity_id, kind, aspect, present } => {
                image.aspect_presence(execution, *entity_id, *kind, aspect)? == Some(*present)
            }
            Self::AbsentField {
                entity_id,
                kind,
                locator,
            } => matches!(
                image.field(execution, *entity_id, *kind, locator)?,
                Some(super::super::observation::WorthQueryApplicationFieldObservation::Absent)
            ),
            Self::OptionalAspectAbsent {
                entity_id,
                kind,
                locator,
            } => matches!(
                image.field(execution, *entity_id, *kind, locator)?,
                Some(super::super::observation::WorthQueryApplicationFieldObservation::AspectAbsent)
            ),
            Self::Relation {
                relation_kind,
                from,
                to,
                matching_relations,
                ..
            } => image.relations(execution, *relation_kind, *from, *to)
            ? == *matching_relations,
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations,
                ..
            } => image.adjacency(execution, *relation_kind, *anchor, *direction, *maximum_work_units)?
                .is_some_and(|current| current == *relations),
            Self::IndexedEntitySelection {
                index_id,
                definition,
                entity_kind,
                locator,
                value,
                candidate_limit,
                candidates,
            } => match applied {
                Some(candidate) => indexed_entity_selection::applied::compare(self, execution, runtime, snapshot, None, candidate)?,
                None => indexed_entity_selection::remains_equal(
                runtime,
                execution,
                snapshot,
                *index_id,
                definition,
                *entity_kind,
                locator,
                value,
                *candidate_limit,
                candidates,
            )?,
            },
            Self::WorkflowDefinitionPredecessor {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *lineage,
                *expected_definition,
                *maximum_work_units,
            )?,
            Self::WorkflowDefinitionCurrent {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                Some(*lineage),
                Some(*expected_definition),
                *maximum_work_units,
            )?,
            Self::WorkflowInstanceCapacity {
                relation_kind,
                lineage,
                maximum_instances,
                instances,
            } => workflow_instance_capacity::remains_equal(
                runtime,
                execution,
                snapshot,
                *relation_kind,
                *lineage,
                *maximum_instances,
                instances,
            )?,
            Self::WorkflowHistoryBasis {
                requirement,
                transition_count,
                snapshot: observed,
                ..
            } => workflow_history_basis::remains_equal(
                runtime,
                snapshot,
                observed,
                *requirement,
                *transition_count,
            ),
        };
        check_request()?;
        Ok(equal)
    }
}
