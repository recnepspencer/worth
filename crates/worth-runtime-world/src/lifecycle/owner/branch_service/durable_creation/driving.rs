//! Drive the acknowledged original; retain each authentic owner result before proceeding.
use super::*;
use crate::branch::{ComponentBranchTarget, OwnerCreatedComponentCustodyRecord};
use crate::durability::WorldDurablePublicationStop as Stop;
use crate::publication::durable_signal::WorldDurableSignalProgress;
use crate::publication::{
    CompositeAttemptProgress, RelationalAttemptProgress, SignalAttemptProgress,
};
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn advance_durable_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        custody: &mut ActiveAttemptCustody,
        body: &mut WorldDurableCreationCustody,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        use RuntimeWorldBranchAdmissionDenial as Denial;
        if body.witness.installed_commit().is_some() {
            self.finish_installed_creation(custody, body);
            return Ok(());
        }
        if body.phase == CreationPhase::Intent {
            match body.binding.publish(&body.subject, execution) {
                Ok(actual) => {
                    body.permit
                        .as_mut()
                        .expect("original source")
                        .acknowledge_intent(actual);
                    body.phase = CreationPhase::Owners;
                }
                Err(Stop::Refused(actual)) => {
                    let retired = body
                        .permit
                        .take()
                        .expect("original source")
                        .proven_no_effect(actual.proof());
                    body.no_effect = Some(
                        crate::lifecycle::RuntimeWorldBranchCreationNoEffect::from_original(actual),
                    );
                    body.phase = CreationPhase::NoEffect;
                    drop(retired);
                    return Err(Denial::OwnerUnavailable);
                }
                Err(Stop::Pending | Stop::Busy) => return Err(Denial::OwnerUnavailable),
                Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                    body.phase = CreationPhase::ReopenRequired;
                    return Err(Denial::OwnerUnavailable);
                }
            }
        }
        if body.phase == CreationPhase::Owners && cancellation.is_cancelled() {
            body.prepare_unsubmitted_release(execution)?;
        }
        if body.phase == CreationPhase::Releasing {
            body.drive_unsubmitted_release(execution)?;
            return Ok(());
        }
        if body.phase == CreationPhase::Owners {
            // Signal seals its original without physical I/O. Record its observer
            // before Native runs or Signal's first physical drive can unwind.
            if let Some(prepared) = body.signal_prepared.take() {
                let attachment = match &self.state.signal {
                    crate::lifecycle::signal_services::RuntimeWorldSignalServices::Authored(
                        actual,
                    ) => actual,
                    crate::lifecycle::signal_services::RuntimeWorldSignalServices::Memory {
                        ..
                    } => unreachable!("Home admission"),
                };
                // An unwind retains the original intent and prevents a second begin.
                body.phase = CreationPhase::ReopenRequired;
                let task = match prepared.begin(
                    *execution,
                    body.permit.as_ref().expect("ACK source"),
                    &attachment.issuer,
                    cancellation.signal_token(),
                ) {
                    Ok(task) => task,
                    Err(actual) => {
                        body.preparation_denial = Some(actual);
                        return Err(Denial::OwnerUnavailable);
                    }
                };
                let task = body
                    .signal_backing
                    .take()
                    .expect("prepaid task")
                    .retain(task);
                body.progress
                    .set_signal(SignalAttemptProgress::unresolved(task.clone()));
                body.signal_task = Some(task);
                body.phase = CreationPhase::Owners;
            }
            if let (Some(target), Some(source)) =
                (body.native_target.take(), body.native_source.take())
            {
                body.phase = CreationPhase::ReopenRequired;
                let actual = self
                    .state
                    .relational_services()
                    .fork_port()
                    .fork_reserved_associated_with_basis(
                        execution,
                        target,
                        source,
                        body.native_caller.take().expect("original association"),
                    );
                match actual {
                    Ok(actual) => {
                        let (outcome, basis, proof) = actual.into_parts();
                        let target = match body.plan.relational() { crate::branch::RelationalBranchCreationPlan::ForkExact { target } => target.clone(), crate::branch::RelationalBranchCreationPlan::ReuseExact => unreachable!("actual fork target") };
                        let identity = outcome.target_identity().clone();
                        body.native_original = Some(proof.consume(|actual| actual));
                        let (_, signal) = body.progress.retained_image().into_parts();
                        body.progress = CompositeAttemptProgress::new(RelationalAttemptProgress::forked(outcome, basis), signal);
                        if let Some(slot) = body.native_custody.take() {
                            slot.install(OwnerCreatedComponentCustodyRecord::new(body.branch.clone(), body.incarnation,
                                ComponentBranchTarget::Relational { target, identity }));
                        }
                        body.phase = CreationPhase::Owners;
                    }
                    Err(worth_relational::facade::branch::RelationalBranchTransitionStop::DurabilityUnresolved(original)) => {
                        let (_, signal) = body.progress.retained_image().into_parts();
                        body.progress = CompositeAttemptProgress::new(RelationalAttemptProgress::durability_unresolved(original), signal);
                        return Err(Denial::OwnerUnavailable);
                    }
                    Err(worth_relational::facade::branch::RelationalBranchTransitionStop::Denied(_)
                        | worth_relational::facade::branch::RelationalBranchTransitionStop::RequestStopped(_)
                        | worth_relational::facade::branch::RelationalBranchTransitionStop::OriginalInputsStopped(_)) => return Err(Denial::OwnerUnavailable),
                }
            }
            if let Some(task) = &body.signal_task {
                match task.drive(execution) {
                    WorldDurableSignalProgress::Committed(actual) => {
                        let outcome = match actual.output.effect() {
                            worth_signal::facade::durability::SignalDurableOwnerEffect::Forked(fork) => fork.result(),
                            worth_signal::facade::durability::SignalDurableOwnerEffect::Advanced(_) => unreachable!("same fork task"),
                        };
                        let reference = outcome
                            .retirement_reference()
                            .expect("owner-issued cleanup reference")
                            .clone();
                        let target = match body.plan.signal() {
                            crate::branch::SignalBranchCreationPlan::ForkExact { target } => {
                                target.clone()
                            }
                            crate::branch::SignalBranchCreationPlan::ReuseExact => {
                                unreachable!("only forks have a task")
                            }
                        };
                        body.progress
                            .set_signal(SignalAttemptProgress::durable(actual));
                        if let Some(slot) = body.signal_custody.take() {
                            slot.install(OwnerCreatedComponentCustodyRecord::new(
                                body.branch.clone(),
                                body.incarnation,
                                ComponentBranchTarget::Signal { target, reference },
                            ));
                        }
                    }
                    WorldDurableSignalProgress::Pending => return Err(Denial::OwnerUnavailable),
                    WorldDurableSignalProgress::ProvenNoEffect
                    | WorldDurableSignalProgress::Indeterminate => {
                        body.phase = CreationPhase::ReopenRequired;
                        return Err(Denial::OwnerUnavailable);
                    }
                }
            }
            custody.record_progress(body.progress.retained_image());
            self.prepare_creation_selection(execution, custody, body)?;
            body.phase = CreationPhase::Prepared;
        }
        if body.phase == CreationPhase::Prepared {
            match body.binding.publish(
                body.selected.as_ref().expect("prepared selection"),
                execution,
            ) {
                Ok(actual) => {
                    body.permit
                        .as_mut()
                        .expect("same source")
                        .acknowledge_disposition(actual);
                    body.phase = CreationPhase::Selected;
                }
                Err(Stop::Pending | Stop::Busy | Stop::Refused(_)) => {
                    return Err(Denial::OwnerUnavailable)
                }
                Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                    body.phase = CreationPhase::ReopenRequired;
                    return Err(Denial::OwnerUnavailable);
                }
            }
        }
        if body.phase == CreationPhase::Selected {
            // Every owner call, encoding and pin acquisition precedes this ACK.
            custody.install_durable_creation_cell(
                &mut body.reservation,
                body.permit.as_mut().expect("same source"),
                &body.source,
                body.branch.clone(),
                body.incarnation,
            );
            self.finish_installed_creation(custody, body);
        }
        match body.phase {
            CreationPhase::Performed | CreationPhase::Released => Ok(()),
            CreationPhase::Intent
            | CreationPhase::Owners
            | CreationPhase::Releasing
            | CreationPhase::Prepared
            | CreationPhase::Selected
            | CreationPhase::NoEffect
            | CreationPhase::ReopenRequired => Err(Denial::OwnerUnavailable),
        }
    }
    fn finish_installed_creation(
        &self,
        custody: &mut ActiveAttemptCustody,
        body: &mut WorldDurableCreationCustody,
    ) {
        let observation = custody.creation_checkpoint_observation();
        if let Some(permit) = body.permit.as_mut() {
            permit.complete_installed_destination(observation.snapshot());
        }
        if let Some(completion) = body.completion.as_mut() {
            completion.completed(&body.source, &observation);
        }
        body.completion = None;
        if let Some(origin) = body.origin.take() {
            body.binding.retain_checkpoint_occurrence(origin);
        }
        body.phase = CreationPhase::Performed;
    }
}
