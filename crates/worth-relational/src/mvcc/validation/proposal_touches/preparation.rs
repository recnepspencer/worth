//! Reached work and returned allocation custody use the original request.
use super::{
    collection::{self, Extent, TouchCollection},
    *,
};
use crate::authority::commit::pipeline::{consult_stage_control, CommitStage};
use crate::transactions::data::{EntityMutationIntent, MutationIntent};
type Error = ValidatedMutationTouchProjectionError;

pub(super) fn project<'p>(
    proposal: &'p ValidatedRelationalProposal,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ValidatedMutationTouches<'p>, Error> {
    request
        .in_scope(|lease| project_in_scope(proposal, request, lease))
        .map_err(|stop| collection::execution(stop.into()))?
}
fn project_in_scope<'p>(
    proposal: &'p ValidatedRelationalProposal,
    request: &ExecutionRequest<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<ValidatedMutationTouches<'p>, Error> {
    consult_stage_control(request, CommitStage::Validation)
        .map_err(|stop| collection::execution(stop.into()))?;
    let (extent, _, _) = run(proposal, request, None)?;
    consult_stage_control(request, CommitStage::Validation)
        .map_err(|stop| collection::execution(stop.into()))?;
    let bytes = extent.bytes()?;
    let memory = match lease {
        Some(lease) => worth_execution::ExecutionAllocationPolicy::Execution(lease)
            .reserve_memory(bytes)
            .map_err(Error::Allocation)?,
        None => Some(
            ExecutionMemoryReservation::reserve_in_scope(None, bytes).map_err(|cause| {
                collection::execution(worth_execution::WorkCeilingDenial::Admission(cause).into())
            })?,
        ),
    };
    consult_stage_control(request, CommitStage::Validation)
        .map_err(|stop| collection::execution(stop.into()))?;
    let backing = extent.allocate(memory)?;
    let (_, backing, mut work) = run(proposal, request, Some(backing))?;
    let mut backing = backing.expect("build pass owns the admitted backing");
    let comparisons = backing
        .touches
        .len()
        .checked_add(backing.structs.len())
        .and_then(|n| n.checked_mul(usize::BITS as usize - n.max(1).leading_zeros() as usize))
        .ok_or_else(collection::capacity)?;
    metered(proposal, request, |checkpoint| {
        checkpoint(comparisons as u64)
    })?;
    canonicalize_structs(&mut backing.structs, &mut backing.touches);
    backing.touches.sort_unstable();
    backing.touches.dedup();
    consult_stage_control(request, CommitStage::Validation)
        .map_err(|stop| collection::execution(stop.into()))?;
    work.mutation_targets_materialized = backing.touches.len() + backing.structs.len();
    Ok(ValidatedMutationTouches {
        backing: Arc::new(backing),
        work,
    })
}
fn run<'p>(
    proposal: &'p ValidatedRelationalProposal,
    request: &ExecutionRequest<'_, '_>,
    backing: Option<TouchBacking<'p>>,
) -> Result<
    (
        Extent,
        Option<TouchBacking<'p>>,
        ValidatedMutationTouchProjectionWork,
    ),
    Error,
> {
    metered(proposal, request, |checkpoint| {
        let mut sink = TouchCollection {
            extent: Extent::default(),
            backing,
            failure: None,
            checkpoint,
        };
        let mut work = ValidatedMutationTouchProjectionWork::default();
        for intent in &proposal.prepared.merged_plan.merged_intents {
            (sink.checkpoint)(1)?;
            work.validated_intents_examined += 1;
            match intent {
                MutationIntent::Entity(intent @ EntityMutationIntent::ApplyAspectPatch(_)) => {
                    super::struct_aspect::project(proposal, intent, &mut sink, &mut work)?
                }
                _ => super::intent_projection::project_intent(
                    intent,
                    &proposal.prepared.working_state,
                    &mut sink,
                    &mut work,
                )?,
            }
            if let Some(failure) = sink.failure.take() {
                return Err(failure);
            }
        }
        Ok((sink.extent, sink.backing, work))
    })
}
fn metered<T>(
    proposal: &ValidatedRelationalProposal,
    request: &ExecutionRequest<'_, '_>,
    run: impl FnOnce(&mut dyn FnMut(u64) -> Result<(), Error>) -> Result<T, Error>,
) -> Result<T, Error> {
    request
        .in_scope(|lease| match lease {
            Some(lease) => crate::execution::prepare_borrowed_artifact(
                lease,
                proposal.commit_work_budget.as_ref(),
                0,
                |budget| {
                    Ok((
                        run(&mut |units| {
                            budget.checkpoint(units).map_err(|error| {
                                collection::execution(crate::execution::kernel_failure(error))
                            })
                        }),
                        0,
                    ))
                },
            )
            .map_err(|stop| Error::Execution(stop.into()))?,
            None => {
                worth_execution::ExecutionWorkCeiling::new(request.work_ceiling())
                    .run_with_checkpoint(None, None, |context| {
                        run(&mut |units| {
                            consult_stage_control(request, CommitStage::Validation)
                                .map_err(|stop| collection::execution(stop.into()))?;
                            context
                                .checkpoint(units)
                                .map_err(|stop| collection::execution(stop.into()))
                        })
                    })
                    .map_err(|stop| collection::execution(stop.into()))?
                    .0
            }
        })
        .map_err(|stop| collection::execution(stop.into()))?
}

// One complete group per exact native entity/aspect. A whole-operation marker
// is unsupported only when the entire group has no prior or supplied fields.
// Duplicate operations for one aspect and duplicate entity intents remain refused
// upstream; this projection never supplies sequence admission authority.
fn canonicalize_structs(
    touches: &mut Vec<ValidatedEntityStructTouch<'_>>,
    unrepresentable: &mut Vec<ValidatedMutationTouch>,
) {
    touches.sort_unstable_by(|a, b| {
        (a.entity, a.contract.key(), a.field).cmp(&(b.entity, b.contract.key(), b.field))
    });
    touches.dedup();
    let mut written = 0;
    for read in 0..touches.len() {
        let item = touches[read];
        let has_fields = item.field.is_none()
            && touches.get(read + 1).is_some_and(|next| {
                next.entity == item.entity
                    && next.contract.key() == item.contract.key()
                    && next.field.is_some()
            });
        if item.field.is_none() {
            if !has_fields {
                unrepresentable.push(ValidatedMutationTouch::UnrepresentableEntityMutation {
                    kind: item.kind,
                });
            }
        } else {
            touches[written] = item;
            written += 1;
        }
    }
    touches.truncate(written);
}
