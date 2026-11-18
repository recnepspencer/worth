//! C issues permission; the candidate may not change that permission's support.
use super::*;
use crate::domain_computation::authorization::SourceAuthorizationLocus as Locus;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial as Stop, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationalAdjacencyDirection as Direction,
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

impl SourceReadAuthorization {
    pub(in crate::domain_computation::primary_graph) fn applied_candidate(
        &self,
    ) -> Option<&RelationalAppliedCandidateReadIdentity> {
        self.applied.as_ref()
    }

    /// Independently granted conventional permission remains C-owned. Compare
    /// its complete actual loci with final D; an own effect cannot grant itself
    /// READ or erase an authorization dependency. This never reads Source from C.
    pub(in crate::domain_computation::primary_graph) fn supports_final_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        complete: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, Stop> {
        let Some(origin) = self.applied_candidate() else {
            return Ok(false);
        };
        if !complete
            .candidate_identity()
            .includes_assembly_prefix(origin)
        {
            return Ok(false);
        }
        self.validate_current(runtime, execution, before)
            .map_err(Stop::output_observation)?;
        let left = runtime
            .read_truth()
            .project_snapshot(before)
            .ok_or_else(unavailable)?;
        let right = complete.project_candidate();
        let physical =
            |cause| Stop::relational_read_stopped(cause, "applied Source authorization support");
        let mut equal = true;
        let conventional = self.visit_loci(&mut |locus| {
            execution.consult().map_err(|cause| physical(cause.into()))?;
            equal &= match locus {
                Locus::Entity(entity) => {
                    let read = |view: &worth_relational::facade::runtime::VisibilityProjectionView| {
                        view.entity_record_with_projection_scope(execution, entity, ProjectionAspectScope::empty(),
                            |record| Some((record.kind_id(), record.lifecycle())))
                    };
                    read(&left).map_err(physical)? == read(&right).map_err(physical)?
                }
                Locus::Relation(relation) => {
                    let read = |view: &worth_relational::facade::runtime::VisibilityProjectionView| {
                        view.relation_record_with_projection_scope(execution, relation, ProjectionAspectScope::empty(),
                            |record| Some((record.kind_id(), record.source(), record.target(), record.lifecycle())))
                    };
                    read(&left).map_err(physical)? == read(&right).map_err(physical)?
                }
                Locus::Field(entity, field) => {
                    // Native policy observations retain the installed Planned
                    // locator. Resolve only that retained observation to its
                    // physical field, as Native durable permission capture does.
                    // The original decision and C currentness stay above.
                    let physical_field;
                    let _storage;
                    let field = if field.aspect().authority() == worth_foundational::facade::LocatorAuthority::Planned {
                        _storage = SourceFactsAllocation::open(execution).map_err(physical)?;
                        let bytes = u64::try_from(field.owned_allocation_capacity_bytes())
                            .map_err(|_| unavailable())?;
                        _storage.claim(execution, bytes).map_err(physical)?;
                        SourceFactsAllocation::work(execution, bytes).map_err(physical)?;
                        physical_field = crate::domain_computation::primary_graph::observations::authoritative(field);
                        &physical_field
                    } else { field };
                    left.compare_entity_field_support(execution, &right, entity, field)
                        .map_err(|cause| match cause {
                            worth_relational::facade::runtime::RelationalFieldSupportComparisonDenial::Execution(cause) => physical(cause),
                            _ => unavailable(),
                        })?
                }
                Locus::Adjacency(anchor, relation, direction) => {
                    let direction = match direction {
                        worth_relational::facade::authorization::RelationalAuthorizationTraversalDirection::Forward => Direction::Outgoing,
                        worth_relational::facade::authorization::RelationalAuthorizationTraversalDirection::Reverse => Direction::Incoming,
                    };
                    left.compare_adjacency_support(execution, &right, anchor, relation, direction)
                        .map_err(|cause| match cause {
                            worth_relational::facade::runtime::RelationalAdjacencySupportDenial::Execution(cause) => physical(cause),
                            worth_relational::facade::runtime::RelationalAdjacencySupportDenial::Allocation(cause) => Stop::allocation_denied("applied Source authorization support", cause),
                            _ => unavailable(),
                        })?
                }
            };
            Ok::<_, Stop>(())
        })?;
        Ok(conventional && equal)
    }
}
fn unavailable() -> Stop {
    Stop::new(
        Kind::MissingAuthoritativeFact,
        "applied Source authorization support",
    )
}
