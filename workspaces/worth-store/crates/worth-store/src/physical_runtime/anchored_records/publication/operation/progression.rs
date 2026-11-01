//! A caller performs one preparation or observes one accepted physical step.
use super::super::super::membership::SelectedRecord;
use super::*;
use crate::physical_runtime::record_serving::{AnchoredRecordAppend, PreparedDurableRecordAppend};
use crate::physical_runtime::{PhysicalMutationOutcome, PhysicalRecordId};
use sha2::{Digest, Sha256};
use std::panic::{catch_unwind, AssertUnwindSafe};

impl AnchoredRecordPublicationOperation {
    pub fn step(
        self,
        records: &AnchoredRecords<'_>,
        custody: &dyn AnchoredRecordStepCustodyPort,
    ) -> AnchoredRecordPublicationStep {
        self.drive(records, Some(custody))
    }

    pub(in crate::physical_runtime::anchored_records) fn drive(
        mut self,
        records: &AnchoredRecords<'_>,
        custody: Option<&dyn AnchoredRecordStepCustodyPort>,
    ) -> AnchoredRecordPublicationStep {
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.shared.require_receiver(records)?;
            if self.shared.state().accepted.is_some() {
                if !settle_accepted(&self.shared, records)? {
                    if self.shared.state().stopped {
                        return Err(AnchoredRecordOperationFailure::Stopped);
                    }
                    return Ok(false);
                }
                return Ok(true);
            }
            if self.shared.state().stopped {
                return Err(AnchoredRecordOperationFailure::Stopped);
            }
            submit_next(&self.shared, records, custody)?;
            Ok(false)
        }));
        match result {
            Ok(Ok(ready)) => {
                let terminal = self.shared.state().terminal.clone();
                if let Some(receipt) = terminal {
                    if let Some(custody) = self.custody.take() {
                        let _receipt = custody.terminate(PublicationDisposition::Published);
                    }
                    AnchoredRecordPublicationStep::Completed(receipt)
                } else if ready {
                    AnchoredRecordPublicationStep::Ready(self)
                } else {
                    AnchoredRecordPublicationStep::Pending(self)
                }
            }
            result => {
                let cause = match result {
                    Ok(Err(cause)) => cause,
                    Err(_) => AnchoredRecordOperationFailure::Panicked,
                    Ok(Ok(_)) => unreachable!(),
                };
                AnchoredRecordPublicationStep::Failed(AnchoredRecordPublicationFailureCustody {
                    stopped: self.stop(),
                    cause,
                })
            }
        }
    }
}

fn submit_next(
    shared: &Arc<PublicationShared>,
    records: &AnchoredRecords<'_>,
    port: Option<&dyn AnchoredRecordStepCustodyPort>,
) -> Result<(), AnchoredRecordOperationFailure> {
    let interruption = shared.state().interruption.clone();
    if let Some(stop) = interruption
        .as_ref()
        .and_then(|view| view.check_live().err())
    {
        return Err(AnchoredRecordOperationFailure::OriginalRequestStopped(stop));
    }
    let (mut plan, index) = {
        let mut state = shared.state();
        (
            state
                .plan
                .take()
                .ok_or(AnchoredRecordOperationFailure::Stopped)?,
            state.written_batches,
        )
    };
    let (append, contents, kind, next_plan, final_selection) = if let Some((append, contents)) =
        plan.members
            .next_batch(index, plan.placement, plan.deadline)
    {
        (
            append,
            contents,
            AnchoredRecordStepKind::Members { batch_index: index },
            Some(plan),
            None,
        )
    } else {
        let prefix = shared.state().prefix.clone();
        plan.locators = plan.resolution.resolve(&prefix);
        let record = plan.header.encode_record(&plan.locators, &plan.payload);
        plan.record_sha256 = Sha256::digest(&record).into();
        let append = AnchoredRecordAppend {
            record,
            anchor: plan.name.anchor(),
            expected_generation: plan.expected_generation,
            idempotency: super::super::material(
                plan.name,
                shared.identity.generation,
                plan.record_sha256,
                shared.identity.runtime.get(),
            ),
            placement: plan.placement,
            deadline: plan.deadline,
        }
        .into_durable_append()
        .map_err(super::super::door_failure)?;
        let selection = super::admission::FinalSelection {
            name: plan.name,
            membership: super::super::super::membership::PreparedValueMembership::new(
                plan.header,
                plan.locators,
            ),
            record_sha256: plan.record_sha256,
        };
        (
            append,
            Vec::new(),
            AnchoredRecordStepKind::Anchor,
            None,
            Some(selection),
        )
    };
    let retired = {
        let mut state = shared.state();
        state.final_selection = final_selection;
        state.kind = kind;
        state.contents = contents;
        if state.stopped {
            next_plan
        } else {
            state.plan = next_plan;
            None
        }
    };
    drop(retired);
    let prepared = append
        .prepare_with_interruption(&records.runtime.record_submission(), interruption)
        .map_err(|failure| append_failure(shared, failure))?;
    let identity = match &prepared {
        PreparedDurableRecordAppend::Prepared(prepared) => prepared.mutation_identity(),
        PreparedDurableRecordAppend::Terminal(fact) => match fact.outcome() {
            PhysicalMutationOutcome::Completed(done) => done.mutation_identity(),
            PhysicalMutationOutcome::ProvenNoEffect(fate) => fate.mutation_identity(),
            PhysicalMutationOutcome::Indeterminate(fate) => fate.mutation_identity(),
        },
    };
    let step = AcceptedStep::new(
        shared.identity,
        kind,
        identity,
        index,
        shared.owner.clone(),
        Arc::downgrade(shared),
    );
    // The exact accepted identity is retained before foreign registration, spawn or delivery.
    let stopped = {
        let mut state = shared.state();
        let stopped = state.stopped;
        state.accepted = Some(step.clone());
        stopped
    };
    if stopped {
        match prepared {
            PreparedDurableRecordAppend::Prepared(prepared) => {
                settle_unstarted(records, prepared, &step)?
            }
            PreparedDurableRecordAppend::Terminal(fact) => step.finish(&fact, None),
        }
        return Err(AnchoredRecordOperationFailure::Stopped);
    }
    if let Some(port) = port {
        let retained = catch_unwind(AssertUnwindSafe(|| step.retain(port)));
        let cause = match retained {
            Ok(Ok(())) => None,
            Ok(Err(cause)) => Some(AnchoredRecordOperationFailure::StepCustody(cause)),
            Err(_) => Some(AnchoredRecordOperationFailure::Panicked),
        };
        if let Some(cause) = cause {
            match prepared {
                PreparedDurableRecordAppend::Prepared(prepared) => {
                    settle_unstarted(records, prepared, &step)?
                }
                PreparedDurableRecordAppend::Terminal(fact) => step.finish(&fact, None),
            }
            return Err(cause);
        }
    }
    match prepared {
        PreparedDurableRecordAppend::Prepared(prepared) => {
            prepared.start_anchored_step(step);
        }
        PreparedDurableRecordAppend::Terminal(fact) => step.finish(&fact, None),
    }
    Ok(())
}

fn settle_unstarted(
    records: &AnchoredRecords<'_>,
    prepared: crate::physical_runtime::PreparedPhysicalMutation,
    step: &AcceptedStep,
) -> Result<(), AnchoredRecordOperationFailure> {
    match records
        .runtime
        .record_submission()
        .cancel_prepared_before_group_seal(prepared)
    {
        crate::physical_runtime::PhysicalPreSealCancellationOutcome::ProvenNoEffect(fate) => step
            .finish(
                &crate::physical_runtime::PhysicalMutationTerminalFact::from_outcome(
                    PhysicalMutationOutcome::ProvenNoEffect(fate),
                ),
                None,
            ),
        crate::physical_runtime::PhysicalPreSealCancellationOutcome::NotCancelled {
            prepared,
            cause,
        } => {
            step.refuse_unstarted(prepared, cause);
            return Err(AnchoredRecordOperationFailure::StepSettlement(
                AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(cause),
            ));
        }
    }
    Ok(())
}

pub(super) fn settle_accepted(
    shared: &PublicationShared,
    records: &AnchoredRecords<'_>,
) -> Result<bool, AnchoredRecordOperationFailure> {
    let step = shared.state().accepted.clone();
    let Some(step) = step else {
        return Ok(true);
    };
    let Some(terminal) = step.terminal() else {
        if let Some(cause) = step.unstarted_refusal_without_attempt() {
            return Err(AnchoredRecordOperationFailure::StepSettlement(
                AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused(cause),
            ));
        }
        return Ok(false);
    };
    let mut state = shared.state();
    state.last_terminal = Some(terminal.clone());
    let completed = match terminal.outcome() {
        PhysicalMutationOutcome::Completed(done) => done,
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            let failure = append_failure_locked(
                &state,
                crate::physical_runtime::record_serving::DurableRecordAppendFailure::ProvenNoEffect(
                    fate.clone(),
                ),
            );
            retire_failed(&mut state);
            return Err(failure.into());
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            let failure = append_failure_locked(
                &state,
                crate::physical_runtime::record_serving::DurableRecordAppendFailure::Indeterminate(
                    fate.clone(),
                ),
            );
            retire_failed(&mut state);
            return Err(failure.into());
        }
    };
    match state.kind {
        AnchoredRecordStepKind::Members { .. } => {
            if completed.persisted_records().len() != state.contents.len() {
                return Err(append_failure_locked(&state, crate::physical_runtime::record_serving::DurableRecordAppendFailure::CompletedWithoutRecords).into());
            }
            let contents = std::mem::take(&mut state.contents);
            state
                .prefix
                .extend(completed.persisted_records().iter().zip(contents).map(
                    |(record, (sha256, length))| {
                        AnchoredRecordMemberLocator::new(
                            PhysicalRecordId::from_persisted(*record),
                            sha256,
                            length,
                        )
                    },
                ));
            state.written_batches += 1;
            state.accepted = None;
        }
        AnchoredRecordStepKind::Anchor => {
            let [record] = completed.persisted_records() else {
                return Err(AnchoredRecordPublicationFailure::CompletedWithoutRecord.into());
            };
            let plan = state
                .final_selection
                .take()
                .ok_or(AnchoredRecordOperationFailure::Stopped)?;
            let membership = plan.membership.bind_selected(SelectedRecord {
                record: *record,
                record_sha256: plan.record_sha256,
                generation: shared.identity.generation,
            });
            let receipt = AnchoredRecordPublished::Published(membership.described(plan.name));
            // Save the actual selected identity before cache insertion or any subsequent work.
            state.terminal = Some(receipt);
            state.live = false;
            state.interruption = None;
            state.accepted = None;
            drop(state);
            records.remember(plan.name, membership);
            if let Some(cause) = terminal.settlement_failure() {
                return Err(AnchoredRecordOperationFailure::StepSettlement(cause));
            }
            return Ok(true);
        }
    }
    if let Some(cause) = terminal.settlement_failure() {
        retire_failed(&mut state);
        return Err(AnchoredRecordOperationFailure::StepSettlement(cause));
    }
    Ok(true)
}
fn append_failure(
    shared: &PublicationShared,
    failure: crate::physical_runtime::record_serving::DurableRecordAppendFailure,
) -> AnchoredRecordOperationFailure {
    append_failure_locked(&shared.state(), failure).into()
}
fn append_failure_locked(
    state: &PublicationState,
    failure: crate::physical_runtime::record_serving::DurableRecordAppendFailure,
) -> AnchoredRecordPublicationFailure {
    match state.kind {
        AnchoredRecordStepKind::Members { .. } => AnchoredRecordPublicationFailure::MemberWrite {
            written_batches: state.written_batches,
            failure: super::super::member_writes::member_failure(failure),
        },
        AnchoredRecordStepKind::Anchor => super::super::door_failure(failure),
    }
}

fn retire_failed(state: &mut PublicationState) {
    state.stopped = true;
    state.live = false;
    state.interruption = None;
    state.plan = None;
    state.accepted = None;
}
