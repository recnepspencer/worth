//! Count actual original deliveries; retain only port-free completion identities.
use super::*;
use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;
use worth_runtime_world::facade::{
    PerformedCompositePublication, ProductBranchObservation, RuntimeWorldOwnerIdentity,
};

#[derive(Default)]
pub(super) struct HomeTransitionCompletion {
    next: u32,
    source: Option<CompletedStepSource>,
}
struct CompletedStepSource {
    world: worth_runtime_world::facade::CompositeCommitIdentity,
    native: RelationalBranchBasisAdmissionIdentity,
}

#[derive(Debug)]
pub enum ApplicationHomeTransitionCompletionDenial {
    Intent(ApplicationHomeTransitionIntentDenial),
    Execution(worth_execution::WorkCeilingDenial),
    Incomplete,
    SourceMismatch,
    OriginalAssignmentMismatch,
}

#[derive(Debug)]
pub struct ApplicationHomeTransitionStepRejected {
    pub(crate) performed: PerformedCompositePublication,
    pub(crate) cause: ApplicationHomeTransitionCompletionDenial,
}
impl ApplicationHomeTransitionStepRejected {
    pub fn reason(&self) -> &ApplicationHomeTransitionCompletionDenial {
        &self.cause
    }
    pub fn into_parts(
        self,
    ) -> (
        PerformedCompositePublication,
        ApplicationHomeTransitionCompletionDenial,
    ) {
        (self.performed, self.cause)
    }
}

pub(crate) fn accept(
    context: &mut HomeContext,
    world: RuntimeWorldOwnerIdentity,
    performed: PerformedCompositePublication,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeTransitionStepRejected> {
    use ApplicationHomeTransitionCompletionDenial as Denial;
    let checked = (|| {
        request.consult().map_err(Denial::Execution)?;
        let intent = context
            .transition
            .committed_intent(&context.descriptor.binding)
            .map_err(Denial::Intent)?
            .ok_or(Denial::Incomplete)?;
        let mut completion = context
            .transition
            .completion
            .lock()
            .unwrap_or_else(|cause| cause.into_inner());
        let original = intent
            .assignments()
            .get(completion.next as usize)
            .ok_or(Denial::Incomplete)?;
        let before = performed.old_product_head();
        let after = performed.new_product_head();
        world
            .admit_incarnation(performed.attempt_identity().owner_identity())
            .map_err(|cause| {
                Denial::Intent(match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        ApplicationHomeTransitionIntentDenial::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => {
                        ApplicationHomeTransitionIntentDenial::ForeignWorld
                    }
                })
            })?;
        if world != performed.attempt_identity().owner_identity()
            || performed.attempt_identity().owner_identity().durable()
                != Some(original.attempt().owner())
            || performed.attempt_identity().ordinal() != original.attempt().ordinal()
            || before.branch_identity() != after.branch_identity()
            || after.branch_identity().name().as_str() != original.branch().name()
            || before.lifecycle_incarnation() != after.lifecycle_incarnation()
            || after.lifecycle_incarnation().ordinal() != original.incarnation().ordinal()
            || intent
                .plan()
                .predecessor()
                .generation()
                .get()
                .checked_add(u64::from(completion.next))
                != Some(before.reference_generation().get())
            || before.reference_generation().get().checked_add(1)
                != Some(after.reference_generation().get())
        {
            return Err(Denial::OriginalAssignmentMismatch);
        }
        match &completion.source {
            Some(source) => {
                if before.selected_commit() != &source.world
                    || before.basis().relational_basis().admission_identity() != &source.native
                {
                    return Err(Denial::SourceMismatch);
                }
            }
            None => {
                let initial = context.transition.publication().ok_or(Denial::Incomplete)?;
                if !before.matches_observation(initial.reserved.world.predecessor()) {
                    return Err(Denial::SourceMismatch);
                }
            }
        }
        let next = completion.next.checked_add(1).ok_or(Denial::Incomplete)?;
        completion.source = Some(CompletedStepSource {
            world: after.selected_commit().clone(),
            native: after
                .basis()
                .relational_basis()
                .admission_identity()
                .clone(),
        });
        completion.next = next;
        Ok(())
    })();
    if let Err(cause) = checked {
        return Err(ApplicationHomeTransitionStepRejected { performed, cause });
    }
    // Canonical history owns the facts. Consuming the genuine delivery closes
    // its product handoff permanently; the completion cursor retains no ports.
    drop(performed.consume());
    Ok(())
}

pub(crate) fn completed(
    context: &HomeContext,
    source: &ProductBranchObservation,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    worth_proof::CompletedHomeProgramTransitionSteps<RelationalBranchBasisAdmissionIdentity>,
    ApplicationHomeTransitionCompletionDenial,
> {
    use ApplicationHomeTransitionCompletionDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    let intent = context
        .transition
        .committed_intent(&context.descriptor.binding)
        .map_err(Denial::Intent)?
        .ok_or(Denial::Incomplete)?;
    let completion = context
        .transition
        .completion
        .lock()
        .unwrap_or_else(|cause| cause.into_inner());
    if completion.next != intent.plan().steps() {
        return Err(Denial::Incomplete);
    }
    let final_source = completion.source.as_ref().ok_or(Denial::Incomplete)?;
    if source.selected_commit() != &final_source.world
        || source.basis().relational_basis().admission_identity() != &final_source.native
    {
        return Err(Denial::SourceMismatch);
    }
    Ok(context.transition.admit_completed_steps(
        crate::descriptor::binding::commitment(intent.successor()),
        intent.plan().commitment(),
        final_source.native.clone(),
    ))
}
