//! Reached work and returned allocation custody use the original request.
use super::{
    collection::{self, Extent, TouchCollection},
    *,
};
use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::authority::mutation::canonical_deltas::data::CanonicalRecordAspectDelta;
use crate::contracts::execution::request_work::RequestWorkBudget;
use crate::contracts::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, MutationIntent,
};
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
type Error = ValidatedMutationTouchProjectionError;

type InverseCreate<'a> =
    dyn Fn(&MutationIntent, &mut dyn FnMut(u64) -> Result<(), Error>) -> Result<bool, Error> + 'a;

pub struct ProjectionInputs<'p, 'control> {
    pub plan: RelationalSemanticPlan<'p>,
    pub before: &'p WorkingState,
    pub after: &'p WorkingState,
    pub schema: &'p RelationalBranchRootSchemaAuthority,
    pub deltas: &'p [CanonicalRecordAspectDelta],
    pub work_budget: Option<&'p RequestWorkBudget>,
    pub consult: &'control dyn Fn() -> Result<(), Error>,
    pub inverse_create: Option<&'control InverseCreate<'control>>,
}

pub fn project<'p>(
    inputs: ProjectionInputs<'p, '_>,
    request: &ExecutionRequest<'_, '_>,
    commands: Option<&'p [MutationIntent]>,
    validate_inverse: impl FnOnce() -> Result<(), Error>,
) -> Result<ValidatedMutationTouches<'p>, Error> {
    request
        .in_scope(|lease| {
            validate_inverse()?;
            project_in_scope(&inputs, request, lease, commands)
        })
        .map_err(|stop| collection::execution(stop.into()))?
}

fn project_in_scope<'p>(
    inputs: &ProjectionInputs<'p, '_>,
    request: &ExecutionRequest<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    commands: Option<&'p [MutationIntent]>,
) -> Result<ValidatedMutationTouches<'p>, Error> {
    (inputs.consult)()?;
    let (extent, _, _) = run(inputs, request, None, commands)?;
    (inputs.consult)()?;
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
    (inputs.consult)()?;
    let backing = extent.allocate(memory)?;
    let (_, backing, mut work) = run(inputs, request, Some(backing), commands)?;
    let mut backing = backing.expect(/* Proof: Only the build pass reaches this branch and it owns Some(admitted backing); the sizing pass exits through its separate branch. */ "build pass owns the admitted backing");
    let comparisons = backing
        .touches
        .len()
        .checked_add(backing.structs.len())
        .and_then(|n| n.checked_mul(usize::BITS as usize - n.max(1).leading_zeros() as usize))
        .ok_or_else(collection::capacity)?;
    metered(inputs, request, |checkpoint| checkpoint(comparisons as u64))?;
    canonicalize_structs(&mut backing.structs, &mut backing.touches);
    backing.touches.sort_unstable();
    backing.touches.dedup();
    (inputs.consult)()?;
    work.mutation_targets_materialized = backing.touches.len() + backing.structs.len();
    super::work_observation::record(work.validated_intents_examined);
    Ok(ValidatedMutationTouches {
        backing: Arc::new(backing),
        work,
    })
}
fn run<'p>(
    inputs: &ProjectionInputs<'p, '_>,
    request: &ExecutionRequest<'_, '_>,
    backing: Option<TouchBacking<'p>>,
    commands: Option<&'p [MutationIntent]>,
) -> Result<
    (
        Extent,
        Option<TouchBacking<'p>>,
        ValidatedMutationTouchProjectionWork,
    ),
    Error,
> {
    metered(inputs, request, |checkpoint| {
        let mut sink = TouchCollection {
            extent: Extent::default(),
            backing,
            failure: None,
            checkpoint,
        };
        let mut work = ValidatedMutationTouchProjectionWork::default();
        if commands.is_none() {
            if let Some(selection) = inputs.plan.selection() {
                (sink.checkpoint)(1)?;
                sink.insert(ValidatedMutationTouch::UnrepresentableEntityMutation {
                    kind: selection.kind(),
                });
                if let Some(failure) = sink.failure.take() {
                    return Err(failure);
                }
            }
        }
        let commands = commands.unwrap_or_else(|| &inputs.plan.commands().merged_intents);
        for intent in commands {
            (sink.checkpoint)(1)?;
            work.validated_intents_examined += 1;
            if let Some(is_inverse) = inputs.inverse_create
                && is_inverse(intent, sink.checkpoint)?
            {
                continue;
            }
            project_intent(inputs, intent, &mut sink, &mut work)?;
        }
        Ok((sink.extent, sink.backing, work))
    })
}

fn project_intent<'p>(
    inputs: &ProjectionInputs<'p, '_>,
    intent: &'p MutationIntent,
    sink: &mut TouchCollection<'p, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), Error> {
    match intent {
        MutationIntent::Entity(intent @ EntityMutationIntent::ApplyAspectPatch(_)) => {
            super::aspect_patch::project(inputs, intent, sink, work)?
        }
        _ => super::intent_projection::project_intent(intent, inputs.before, sink, work)?,
    }
    if let Some(failure) = sink.failure.take() {
        return Err(failure);
    }
    Ok(())
}

fn metered<T>(
    inputs: &ProjectionInputs<'_, '_>,
    request: &ExecutionRequest<'_, '_>,
    run: impl FnOnce(&mut dyn FnMut(u64) -> Result<(), Error>) -> Result<T, Error>,
) -> Result<T, Error> {
    request
        .in_scope(|lease| match lease {
            Some(lease) => {
                crate::contracts::execution::packet_preparation::prepare_borrowed_artifact(
                    lease,
                    inputs.work_budget,
                    0,
                    |budget| {
                        Ok((
                            run(&mut |units| {
                                budget.checkpoint(units).map_err(|error| {
                                    collection::execution(
                                        crate::contracts::execution::denial::kernel_failure(error),
                                    )
                                })
                            }),
                            0,
                        ))
                    },
                )
                .map_err(|stop| Error::Execution(stop.into()))?
            }
            None => {
                worth_execution::ExecutionWorkCeiling::new(request.work_ceiling())
                    .run_with_checkpoint(None, None, |context| {
                        run(&mut |units| {
                            (inputs.consult)()?;
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
