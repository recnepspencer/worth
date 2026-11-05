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
            match fact {
                Fact::Aggregate { observation } => {
                    for member in std::iter::once(observation.key().member())
                        .chain(observation.native().members())
                    {
                        if !owns(member)? {
                            return Ok(None);
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
                    observation: observation.refresh_at(&pin, execution).map_err(
                        |stop| match stop.kind {
                            PinnedAggregateReadDenialKind::Read(cause) => Failure::execution(cause),
                            _ => Failure::native_unavailable(),
                        },
                    )?,
                },
                Fact::StructuralWalk { observation } => Fact::StructuralWalk {
                    observation: observation.refresh_at(&pin, execution, context).map_err(
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
