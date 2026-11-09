//! An admitted writer's complete predecessor reads become actual output postconditions.
use super::{RebaseVerificationFailure as Failure, WorthQueryApplicationObservedFact as Fact};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    mvcc::RelationalRequiredOutputPublication,
    runtime::{
        PinnedAggregateReadDenialKind, PinnedStructuralWalkDenialKind,
        RelationalPinnedReadAdmissionDenial, RelationalRuntime,
    },
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
pub(super) fn refresh(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    fact: &Fact,
    predecessor: &RelationalRequiredOutputPublication,
    retired: &std::collections::BTreeSet<worth_relational::facade::identity::EntityId>,
) -> Result<Option<Fact>, Failure> {
    if !matches!(fact, Fact::Aggregate { .. } | Fact::StructuralWalk { .. }) {
        return Ok(None);
    }
    let answer = execution
        .run_native_with_checkpoint(|context| {
            let owns = |record| {
                predecessor
                    .contains_output_entity(execution, record)
                    .map_err(Failure::execution)
            };
            let mut aggregate_member = None;
            let mut walk_roots = None;
            match fact {
                Fact::Aggregate { observation } => {
                    for member in std::iter::once(observation.key().member())
                        .chain(observation.native().members())
                    {
                        if !owns(member)? {
                            return Ok(None);
                        }
                        if aggregate_member.is_none() && !retired.contains(&member) {
                            aggregate_member = Some(member);
                        }
                    }
                }
                Fact::StructuralWalk { observation } => {
                    for member in observation.key().roots().iter().copied().chain(
                        observation
                            .native()
                            .visits()
                            .iter()
                            .map(|visit| visit.entity.entity_id),
                    ) {
                        if !owns(member)? {
                            return Ok(None);
                        }
                    }
                    // Native has admitted the complete predecessor's incident
                    // provenance before publication. With every visited entity
                    // covered, its edges need no second ownership authority.
                    if observation.key().roots().iter().any(|root| retired.contains(root)) {
                        // Retain every surviving visited entity as a root. A
                        // partial shrink can split a walk; picking only one
                        // survivor could lose another owned region's support.
                        let visits = observation.native().visits();
                        let bytes = visits.len().checked_mul(size_of::<worth_relational::facade::identity::EntityId>())
                            .and_then(|bytes| u64::try_from(bytes).ok())
                            .ok_or_else(|| Failure::execution(Cause::ChargedBytesOverflow))?;
                        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
                            execution, context.has_accounting(), bytes,
                        ).map_err(Failure::execution)?;
                        let mut roots = Vec::new();
                        roots.try_reserve_exact(visits.len()).map_err(|_| Failure::execution(Cause::AllocationFailed))?;
                        for visit in visits {
                            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
                                .map_err(|stop| Failure::execution(stop.into()))?;
                            let entity = visit.entity.entity_id;
                            if !retired.contains(&entity) { roots.push(entity); }
                        }
                        if roots.is_empty() { return Err(Failure::native_unavailable()); }
                        walk_roots = Some((roots, memory));
                    }
                }
                _ => unreachable!(),
            }
            let pin = runtime
                .read_truth()
                .pin_read_capability(snapshot, &[], execution)
                .map_err(|stop| match stop {
                    RelationalPinnedReadAdmissionDenial::Request(cause) => {
                        Failure::execution(cause)
                    }
                    _ => Failure::native_unavailable(),
                })?;
            let fact = match fact {
                Fact::Aggregate { observation } => Fact::Aggregate {
                    // The old selector can retire in a partial owned shrink.
                    // Every observed member above belongs to this predecessor;
                    // acquire the actual surviving aggregate at this commit.
                    observation: observation
                        .refresh_at_member(
                            &pin,
                            execution,
                            aggregate_member.ok_or_else(Failure::native_unavailable)?,
                        )
                        .map_err(|stop| match stop.kind {
                            PinnedAggregateReadDenialKind::Read(cause) => Failure::execution(cause),
                            _ => Failure::native_unavailable(),
                        })?,
                },
                Fact::StructuralWalk { observation } => Fact::StructuralWalk {
                    observation: observation.refresh_at_roots(
                        &pin,
                        execution,
                        walk_roots.as_ref().map_or(observation.key().roots(), |(roots, _)| roots.as_slice()),
                        context,
                    ).map_err(
                        |stop| match stop.kind {
                            PinnedStructuralWalkDenialKind::Read(cause) => {
                                Failure::execution(cause)
                            }
                            _ => Failure::native_unavailable(),
                        },
                    )?,
                },
                _ => unreachable!(),
            };
            Ok(Some(fact))
        })
        .map_err(|cause| Failure::execution(Cause::from(cause)))?;
    answer.0
}
