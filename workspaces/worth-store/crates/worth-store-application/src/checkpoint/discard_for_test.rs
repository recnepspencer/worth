//! D14 sealed certification door: discards every durable `RetentionClass::Derived`
//! artifact in the Home (`home.checkpoint` and its anchored members) so reopen
//! reconstructs strictly from authoritative journals and recomputes derived state
//! only on demand.
use super::owner::HomeCheckpointOwner;
use super::ApplicationHomeCheckpointDenial as Denial;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionRequest, ExecutionResourceLease, MapKernelContext, WorkCeilingDenial};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::*;

struct DiscardStepProtection;
impl AnchoredRecordSelectionProtection for DiscardStepProtection {}

struct DiscardStepPort {
    registration: Arc<Mutex<Option<AnchoredRecordStepRegistration>>>,
}

struct DiscardTerminalDestination {
    registration: Arc<Mutex<Option<AnchoredRecordStepRegistration>>>,
}

impl AnchoredRecordStepCustodyPort for DiscardStepPort {
    fn retain_step(
        &self,
        registration: AnchoredRecordStepRegistration,
    ) -> Result<AnchoredRecordStepRetention, AnchoredRecordStepCustodyRefusal> {
        *self
            .registration
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(registration);
        Ok(AnchoredRecordStepRetention::new(
            Box::new(DiscardStepProtection),
            Arc::new(DiscardTerminalDestination {
                registration: Arc::clone(&self.registration),
            }),
        ))
    }
}

impl AnchoredRecordStepTerminalDestination for DiscardTerminalDestination {
    fn record_terminal(
        &self,
        terminal: &AnchoredRecordStepTerminal,
    ) -> Result<(), AnchoredRecordStepDeliveryFailure> {
        let guard = self
            .registration
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let registration = guard
            .as_ref()
            .ok_or(AnchoredRecordStepDeliveryFailure::ReceiptRefused)?;
        if registration.mutation_identity() != terminal.mutation_identity() {
            return Err(AnchoredRecordStepDeliveryFailure::ReceiptRefused);
        }
        Ok(())
    }
}

pub(crate) fn discard(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<u64, Denial> {
    if let Some(attempt) = owner.pending() {
        return Err(Denial::RecoveryRequired(attempt.observation()));
    }
    execution.consult().map_err(Denial::Execution)?;
    let (_, raw_generation) = super::read_with_anchor_generation(serving, execution)?;
    let mut returned = None;
    let mut run = |lease: Option<&ExecutionResourceLease<'_>>| {
        execution
            .run_native_with_checkpoint(|context| {
                returned = Some(run_discard(
                    serving,
                    owner,
                    raw_generation,
                    context,
                    lease,
                ));
            })
            .map(|_| ())
    };
    let scope = if execution.is_leased() {
        execution.in_scope(run).and_then(|result| result)
    } else {
        run(None)
    };
    match scope {
        Ok(()) => returned.expect("completed discard recorded its result"),
        Err(cause) => match returned {
            Some(Err(denial)) => Err(denial),
            _ => Err(Denial::Execution(cause)),
        },
    }
}

fn run_discard(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    previous_generation: u64,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<u64, Denial> {
    let new_generation =
        publish_discard_marker(serving, owner, previous_generation, context, lease)?;
    // Once the empty discard marker is selected on `home.checkpoint`, durable
    // derived state is discarded; prevent graceful close from re-capturing it.
    owner.mark_derived_discarded();
    run_physical_checkpoint(serving, new_generation, context, lease)?;
    Ok(new_generation)
}

fn publish_discard_marker(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    previous_generation: u64,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<u64, Denial> {
    let view = context
        .retain_interruption_view(lease)
        .map_err(Denial::InterruptionView)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(120_000)
        .expect("fixed positive physical policy");
    let publication = AnchoredRecordPublication::new(
        AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
            .expect("static checkpoint anchor"),
        previous_generation,
        super::manifest::DERIVED_DISCARDED_PAYLOAD.to_vec(),
        crate::open::physical_admission::placement(),
        deadline,
    )
    .with_members(Vec::new());
    let mut operation = match records.start_publication(publication, view) {
        AnchoredRecordPublicationStart::Ready { operation, .. } => operation,
        AnchoredRecordPublicationStart::AlreadyPublished(receipt) => {
            return Ok(published_generation(receipt));
        }
        AnchoredRecordPublicationStart::Refused(cause) => {
            return Err(Denial::PhysicalPublication(cause));
        }
    };
    let port = DiscardStepPort {
        registration: Arc::new(Mutex::new(None)),
    };
    loop {
        if let Err(stop) = context.poll() {
            let mut stopped = operation.stop();
            loop {
                match stopped.settle(&records) {
                    AnchoredRecordStoppedPoll::Pending(next) => {
                        stopped = next;
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    AnchoredRecordStoppedPoll::Settled(observation) => {
                        if observation.terminal().is_some() {
                            owner.mark_derived_discarded();
                        }
                        break;
                    }
                    AnchoredRecordStoppedPoll::Failed(_) => break,
                }
            }
            return Err(Denial::Execution(WorkCeilingDenial::Stopped(stop)));
        }
        match operation.step(&records, &port) {
            AnchoredRecordPublicationStep::Pending(next)
            | AnchoredRecordPublicationStep::Ready(next) => {
                operation = next;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            AnchoredRecordPublicationStep::Completed(receipt) => {
                return Ok(published_generation(receipt));
            }
            AnchoredRecordPublicationStep::Failed(failure) => {
                return Err(Denial::Publication(
                    AnchoredRecordPublicationFailure::OwnedPublication(Box::new(failure)),
                ));
            }
        }
    }
}

fn published_generation(receipt: AnchoredRecordPublished) -> u64 {
    let (AnchoredRecordPublished::Published(membership)
    | AnchoredRecordPublished::AlreadyPublished(membership)) = receipt;
    membership.version().generation()
}

fn run_physical_checkpoint(
    serving: &ServingPhysicalRuntime,
    generation: u64,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Denial> {
    let view = context
        .retain_interruption_view(lease)
        .map_err(Denial::InterruptionView)?;
    let mut key = [0_u8; 32];
    key[..8].copy_from_slice(b"WAPHDISC");
    key[8..16].copy_from_slice(&generation.to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(120_000)
            .expect("fixed positive physical policy"),
    );
    let handle = match serving
        .checkpoints()
        .start_with_interruption_view(request, view)
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        other => {
            return Err(Denial::PhysicalCheckpoint {
                detail: format!(
                    "derived discard physical checkpoint start {}",
                    crate::open::physical_admission::unfinished_outcome(&other)
                ),
            });
        }
    };
    loop {
        match handle.poll() {
            PhysicalCheckpointPoll::Pending(_) => {
                context
                    .poll()
                    .map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))?;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            PhysicalCheckpointPoll::Terminal(PhysicalCheckpointOutcome::Completed(_)) => {
                return Ok(());
            }
            PhysicalCheckpointPoll::Terminal(
                outcome @ (PhysicalCheckpointOutcome::ProvenNoEffect(_)
                | PhysicalCheckpointOutcome::Indeterminate(_)),
            ) => {
                return Err(Denial::PhysicalCheckpointOutcome(outcome));
            }
        }
    }
}
