//! D14 sealed certification door: discards every durable `RetentionClass::Derived`
//! artifact in the Home (`home.checkpoint` and its anchored members) by selecting
//! a zero-member discard marker on `home.checkpoint` under the Home's serialized
//! checkpoint owner slot.
use super::owner::{DiscardSlotGuard, HomeCheckpointOwner};
use super::ApplicationHomeCheckpointDenial as Denial;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use worth_execution::{ExecutionRequest, ExecutionResourceLease, MapKernelContext, WorkCeilingDenial};
use worth_store::physical_runtime::*;

static DISCARD_ATTEMPT_NONCE: AtomicU64 = AtomicU64::new(1);

struct DiscardStepProtection;
impl AnchoredRecordSelectionProtection for DiscardStepProtection {}

struct DiscardStepPort {
    registration: Arc<Mutex<Option<AnchoredRecordStepRegistration>>>,
    #[cfg(test)]
    fail_terminal_delivery: bool,
}

struct DiscardTerminalDestination {
    registration: Arc<Mutex<Option<AnchoredRecordStepRegistration>>>,
    #[cfg(test)]
    fail_terminal_delivery: bool,
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
                #[cfg(test)]
                fail_terminal_delivery: self.fail_terminal_delivery,
            }),
        ))
    }
}

impl AnchoredRecordStepTerminalDestination for DiscardTerminalDestination {
    fn record_terminal(
        &self,
        terminal: &AnchoredRecordStepTerminal,
    ) -> Result<(), AnchoredRecordStepDeliveryFailure> {
        #[cfg(test)]
        if self.fail_terminal_delivery {
            return Err(AnchoredRecordStepDeliveryFailure::ReceiptRefused);
        }
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
    discard_impl(
        serving,
        owner,
        execution,
        #[cfg(test)]
        false,
    )
}

#[cfg(test)]
pub(crate) fn discard_with_terminal_delivery_failure_for_test(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<u64, Denial> {
    discard_impl(serving, owner, execution, true)
}

fn discard_impl(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
    #[cfg(test)] fail_terminal_delivery: bool,
) -> Result<u64, Denial> {
    // Serialize against checkpoint installation, acknowledgement, and another discard.
    let guard = owner.begin_discard()?;
    execution.consult().map_err(Denial::Execution)?;
    let (_, raw_generation) = super::read_with_anchor_generation(serving, execution)?;
    let mut guard = Some(guard);
    let mut returned = None;
    let mut run = |lease: Option<&ExecutionResourceLease<'_>>| {
        execution
            .run_native_with_checkpoint(|context| {
                let slot = guard.take().expect("exclusive discard slot guard");
                returned = Some(publish_discard_marker(
                    serving,
                    slot,
                    raw_generation,
                    context,
                    lease,
                    #[cfg(test)]
                    fail_terminal_delivery,
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

fn publish_discard_marker(
    serving: &ServingPhysicalRuntime,
    guard: DiscardSlotGuard<'_>,
    previous_generation: u64,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    #[cfg(test)] fail_terminal_delivery: bool,
) -> Result<u64, Denial> {
    let view = context
        .retain_interruption_view(lease)
        .map_err(Denial::InterruptionView)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(120_000)
        .expect("fixed positive physical policy");
    let nonce = DISCARD_ATTEMPT_NONCE.fetch_add(1, Ordering::Relaxed);
    let publication = AnchoredRecordPublication::new(
        AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
            .expect("static checkpoint anchor"),
        previous_generation,
        super::manifest::encode_derived_discarded(nonce).to_vec(),
        crate::open::physical_admission::placement(),
        deadline,
    )
    .with_members(Vec::new());
    let mut operation = match records.start_publication(publication, view) {
        AnchoredRecordPublicationStart::Ready { operation, .. } => operation,
        AnchoredRecordPublicationStart::AlreadyPublished(receipt) => {
            let generation = published_generation(receipt);
            guard.finish(true);
            return Ok(generation);
        }
        AnchoredRecordPublicationStart::Refused(cause) => {
            return Err(Denial::PhysicalPublication(cause));
        }
    };
    let port = DiscardStepPort {
        registration: Arc::new(Mutex::new(None)),
        #[cfg(test)]
        fail_terminal_delivery,
    };
    loop {
        if let Err(stop) = context.poll() {
            let mut stopped = operation.stop();
            stopped.request_cancellation();
            loop {
                match stopped.settle(&records) {
                    AnchoredRecordStoppedPoll::Pending(next) => {
                        stopped = next;
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    AnchoredRecordStoppedPoll::Settled(observation) => {
                        guard.finish(observation.terminal().is_some());
                        return Err(Denial::Execution(WorkCeilingDenial::Stopped(stop)));
                    }
                    AnchoredRecordStoppedPoll::Failed(failure) => {
                        return Err(reconcile_failure(guard, failure));
                    }
                }
            }
        }
        match operation.step(&records, &port) {
            AnchoredRecordPublicationStep::Pending(next)
            | AnchoredRecordPublicationStep::Ready(next) => {
                operation = next;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            AnchoredRecordPublicationStep::Completed(receipt) => {
                let generation = published_generation(receipt);
                guard.finish(true);
                return Ok(generation);
            }
            AnchoredRecordPublicationStep::Failed(failure) => {
                return Err(reconcile_failure(guard, failure));
            }
        }
    }
}

fn reconcile_failure(
    guard: DiscardSlotGuard<'_>,
    failure: AnchoredRecordPublicationFailureCustody,
) -> Denial {
    let observation = failure.observe();
    if observation.terminal().is_some() {
        // Zero-member discard marker is already durably selected on home.checkpoint
        // even though terminal settlement failed afterwards. Atomically update live
        // owner state while preserving the typed failure for the caller.
        guard.finish(true);
        return Denial::Publication(AnchoredRecordPublicationFailure::OwnedPublication(Box::new(
            failure,
        )));
    }
    let proven_no_effect = observation.step_terminal().is_some_and(|terminal| {
        terminal.settlement_failure().is_none()
            && matches!(terminal.outcome(), PhysicalMutationOutcome::ProvenNoEffect(_))
    });
    if proven_no_effect {
        guard.finish(false);
        return Denial::Publication(AnchoredRecordPublicationFailure::OwnedPublication(Box::new(
            failure,
        )));
    }
    // Selection is unknown: retain failure custody on the checkpoint slot instead
    // of releasing it as Idle and assuming no effect.
    guard.finish_unresolved(failure);
    Denial::DiscardUnresolved
}

fn published_generation(receipt: AnchoredRecordPublished) -> u64 {
    let (AnchoredRecordPublished::Published(membership)
    | AnchoredRecordPublished::AlreadyPublished(membership)) = receipt;
    membership.version().generation()
}
