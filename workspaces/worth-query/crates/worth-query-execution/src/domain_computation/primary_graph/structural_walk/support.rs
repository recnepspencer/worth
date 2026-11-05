//! Compare real post-own support, including Native retirement of obsolete roots.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{
    PinnedStructuralWalkDenialKind as Denial, PinnedStructuralWalkScope, RelationalPinnedRead,
};

impl StructuralWalkObservation {
    pub(in crate::domain_computation::primary_graph) fn support_equal_between<A, B>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        left: &RelationalPinnedRead<A>,
        right: &RelationalPinnedRead<B>,
    ) -> Result<bool, Failure> {
        let answer = execution
            .run_native_with_checkpoint(|context| {
                let bytes = self
                    .query
                    .roots()
                    .len()
                    .checked_mul(std::mem::size_of::<EntityId>())
                    .ok_or(Failure::Execution(Cause::ChargedBytesOverflow))?;
                let _memory =
                    super::memory::reserve(execution, execution.has_accounting(), bytes as u64)
                        .map_err(Failure::Execution)?;
                let mut roots = Vec::new();
                roots
                    .try_reserve_exact(self.query.roots().len())
                    .map_err(|_| Failure::Execution(Cause::AllocationFailed))?;
                let a = left.project_read();
                let b = right.project_read();
                for root in self.query.roots() {
                    Self::checkpoint(context).map_err(Failure::Execution)?;
                    let before = a
                        .read_entity_retirement(execution, *root)
                        .map_err(Failure::Execution)?;
                    let after = b
                        .read_entity_retirement(execution, *root)
                        .map_err(Failure::Execution)?;
                    match (before, after) {
                        (Some(a), Some(b))
                            if a.kind_id() == b.kind_id()
                                && a.created_at_version() == b.created_at_version()
                                && a.deleted_at_version() == b.deleted_at_version() => {}
                        (None, None) => roots.push(*root),
                        _ => return Ok(false),
                    }
                }
                if roots.is_empty() {
                    return Ok(true);
                }
                let scope = || {
                    PinnedStructuralWalkScope::new(
                        &roots,
                        self.query.entity_kind(),
                        self.query.relation_kind(),
                        self.query.direction(),
                    )
                };
                let left = left
                    .complete_relation_walk(execution, scope())
                    .map_err(support_stop)?;
                let right = right
                    .complete_relation_walk(execution, scope())
                    .map_err(support_stop)?;
                Self::same_native(&left, &right, || {
                    Self::checkpoint(context).map_err(Failure::Execution)
                })
            })
            .map_err(|cause| Failure::Execution(cause.into()))?;
        answer.0
    }
}
fn support_stop(stop: worth_relational::facade::runtime::PinnedStructuralWalkDenial) -> Failure {
    match stop.kind {
        Denial::Read(cause) => Failure::Execution(cause),
        Denial::WorkLimitExceeded | Denial::DistanceOverflow => Failure::WorkBudgetExceeded,
        _ => Failure::Unavailable,
    }
}
