//! Drive retained originals; neither a retry nor a stopped request reexecutes a callback.
use super::task::{SignalDurableAdvanceAnswer, SignalDurableAdvanceDriver};
use super::*;
use crate::branch::owner_services::SignalDurableBranchHandle;
use crate::durability::{
    publication_binding::{admit_definition, SignalPublicationBindingDenial},
    SignalPublicationObservation,
};
use worth_execution::ExecutionRequest;

impl From<SignalPublicationBindingDenial> for SignalDurableAdvanceDenial {
    fn from(denial: SignalPublicationBindingDenial) -> Self {
        match denial {
            SignalPublicationBindingDenial::Medium(denial) => Self::Medium(denial),
            SignalPublicationBindingDenial::WrongCommittedSubject {
                expected_journal,
                expected_digest,
                observed,
            } => Self::WrongCommittedSubject {
                expected_journal,
                expected_digest,
                observed,
            },
            SignalPublicationBindingDenial::DefinitionSourceMismatch {
                expected,
                expected_branch,
                observed,
                observed_branch,
            } => Self::DefinitionSourceMismatch {
                expected,
                expected_branch,
                observed,
                observed_branch,
            },
        }
    }
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn handle(
        self: &Arc<Self>,
        original: SignalBranchAdvanceIdentity,
    ) -> SignalDurableAdvanceTask {
        let driver: Arc<dyn SignalDurableAdvanceDriver> = self.clone();
        SignalDurableAdvanceTask {
            original,
            driver: Arc::downgrade(&driver),
        }
    }
    fn finish(
        &self,
        original: SignalBranchAdvanceIdentity,
        slot: &mut TaskSlot<D, I, T>,
        completion: SignalDurableAdvanceCompletion,
    ) -> SignalDurableAdvanceAnswer {
        let terminal = std::mem::replace(
            slot,
            TaskSlot {
                branch: slot.branch,
                resource: None,
                phase: TaskPhase::Delivered,
                _memory: None,
            },
        );
        SignalDurableAdvanceAnswer::Finished(self.finish_resource(original, terminal, completion))
    }
    fn refused(
        &self,
        original: SignalBranchAdvanceIdentity,
        slot: &mut TaskSlot<D, I, T>,
        fate: DurablePublicationFate<SignalPublicationBindingDenial>,
    ) -> SignalDurableAdvanceAnswer {
        let completion = match fate {
            DurablePublicationFate::Committed => {
                unreachable!("binding success returns its concrete committed subject")
            }
            DurablePublicationFate::ProvenNoEffect { detail } => {
                SignalDurableAdvanceCompletion::proven_no_effect(original, detail.into())
            }
            DurablePublicationFate::Indeterminate { detail } => {
                SignalDurableAdvanceCompletion::indeterminate(original, detail.into())
            }
        };
        self.finish(original, slot, completion)
    }
}
impl<D, I, T> SignalDurableAdvanceDriver for SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn drive_original(
        &self,
        original: SignalBranchAdvanceIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDurableAdvanceAnswer, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        if let Some(original) = *self
            .unsettled
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
        {
            return Err(Dn::PublicationUnsettled { original });
        }
        let task = self.task(original)?;
        let mut slot = match task.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::WouldBlock) => return Err(Dn::TaskDriving { original }),
            Err(std::sync::TryLockError::Poisoned(_)) => return Err(Dn::PublicationStatePoisoned),
        };
        match &slot.phase {
            TaskPhase::Preparing => return Err(Dn::TaskExecuting { original }),
            TaskPhase::Completing => return Err(Dn::TaskCompleting { original }),
            TaskPhase::Delivered => return Err(Dn::CompletionDelivered { original }),
            TaskPhase::Completion(_) => {
                let TaskPhase::Completion(completion) =
                    std::mem::replace(&mut slot.phase, TaskPhase::Completing)
                else {
                    unreachable!("one guard owns the retained completion")
                };
                return Ok(self.finish(original, &mut slot, *completion));
            }
            TaskPhase::Definition(_) | TaskPhase::Branch(_) => {}
        }
        let owner = self.owner()?;
        let admission = owner
            .admit()
            .map_err(super::super::mutation_port::map_advance_admission_denial)
            .map_err(Dn::Mutation)?;
        let cell = owner
            .lookup_cell(&admission, slot.branch)
            .map_err(|denial| {
                super::super::mutation_port::map_advance_registry_denial(denial, slot.branch)
            })
            .map_err(Dn::Mutation)?;
        {
            let mut turn = self
                .physical_turn
                .lock()
                .map_err(|_| Dn::PublicationStatePoisoned)?;
            match *turn {
                Some(current) if current != original => {
                    return Ok(SignalDurableAdvanceAnswer::Pending)
                }
                Some(_) => {}
                None => *turn = Some(original),
            }
        }
        // The adapter must inspect an existing original task even if this
        // request has stopped. Only the adapter decides physical absence.
        if let TaskPhase::Definition(pending) = &slot.phase {
            match self
                .medium
                .advance_definition(&pending.definition, execution)
            {
                SignalPublicationObservation::Pending => {
                    return Ok(SignalDurableAdvanceAnswer::Pending)
                }
                SignalPublicationObservation::Finished(completion) => {
                    let TaskPhase::Definition(pending) =
                        std::mem::replace(&mut slot.phase, TaskPhase::Completing)
                    else {
                        unreachable!("one task guard owns this definition phase")
                    };
                    let DefinitionPending {
                        advance,
                        definition,
                        location,
                    } = *pending;
                    match admit_definition(definition, completion) {
                        Ok(proof) => {
                            let definition = self
                                .source
                                .lock()
                                .expect("source index has no user code")
                                .record_committed_definition(proof, location);
                            slot.phase = TaskPhase::Branch(Box::new(BranchPending {
                                advance,
                                definition,
                            }));
                        }
                        Err(fate) => {
                            drop(advance);
                            return Ok(self.refused(original, &mut slot, fate));
                        }
                    }
                }
            }
        }
        let TaskPhase::Branch(pending) = &slot.phase else {
            unreachable!("a committed definition admits the branch phase")
        };
        let completion = match self
            .medium
            .advance_branch(&pending.advance.artifact, execution)
        {
            SignalPublicationObservation::Pending => {
                return Ok(SignalDurableAdvanceAnswer::Pending)
            }
            SignalPublicationObservation::Finished(completion) => completion,
        };
        let TaskPhase::Branch(pending) = std::mem::replace(&mut slot.phase, TaskPhase::Completing)
        else {
            unreachable!("one task guard owns this branch phase")
        };
        let BranchPending {
            advance,
            definition,
        } = *pending;
        let ReadyAdvance {
            prepared,
            artifact,
            retention,
            location,
            publication,
        } = advance;
        match medium_binding::admit_advance(prepared, artifact, definition, completion) {
            Ok(proof) => {
                let (old_state, transaction, observation, record) =
                    cell.commit_durable_advance(&admission, proof);
                let artifact = self
                    .source
                    .lock()
                    .expect("source index has no user code")
                    .record_committed_advance(record, location);
                let basis = owner.admit_canonical_basis(
                    observation,
                    slot.branch,
                    cell.incarnation().get(),
                    retention.into_one(),
                );
                let branch = SignalDurableBranchHandle::from_owner(
                    basis,
                    artifact.descriptor().clone(),
                    self.medium.binding().incarnation().fence(),
                );
                use worth_foundational::facade::SignalDefinitionProjectionOutcome as Projection;
                let memory = slot._memory.take();
                drop(slot);
                let mut output = SignalDurableAdvanceOutput {
                    projection: Projection::NotRequired,
                    branch,
                    artifact,
                    transaction: SignalDurableTransactionResult {
                        result: transaction,
                        _memory: memory,
                    },
                };
                let had_publication = publication.is_some();
                let mut projection_done = false;
                // No task/cell/source-index mutex spans a producer or destructor.
                // Retain the known committed result even if either unwinds.
                let cleanup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if let Some(publication) = publication {
                        publication.committed(output.branch());
                        projection_done = true;
                    }
                    drop(old_state);
                }));
                output.projection = match (had_publication, projection_done) {
                    (false, false) | (false, true) => Projection::NotRequired,
                    (true, true) => Projection::Installed,
                    (true, false) => Projection::UnwoundAfterCommit,
                };
                let completion = SignalDurableAdvanceCompletion::committed(original, output);
                let mut slot = task
                    .lock()
                    .expect("one original owns its retained completion");
                slot.phase = TaskPhase::Completion(Box::new(completion));
                if let Err(payload) = cleanup {
                    drop(slot);
                    std::panic::resume_unwind(payload);
                }
                let TaskPhase::Completion(completion) =
                    std::mem::replace(&mut slot.phase, TaskPhase::Completing)
                else {
                    unreachable!("original completion was just installed")
                };
                Ok(self.finish(original, &mut slot, *completion))
            }
            Err((prepared, artifact, fate)) => {
                drop((prepared, artifact, retention));
                Ok(self.refused(original, &mut slot, fate))
            }
        }
    }
}
