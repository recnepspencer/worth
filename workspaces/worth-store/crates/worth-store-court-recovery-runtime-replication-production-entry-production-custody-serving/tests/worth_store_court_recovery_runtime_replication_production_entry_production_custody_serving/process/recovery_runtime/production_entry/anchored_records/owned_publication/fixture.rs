//! Original input tickets and real Store gates; no Native authority substitute.
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use worth_execution::{
    CancellationSource, ExecutionInterruptionView, ExecutionMemoryReservation, ExecutionRequest,
    SerialMemoryBudget, SerialRequest,
};
use worth_store::physical_runtime::production::{
    PhysicalMutationCheckpoint, PhysicalMutationPauseGate,
};
use worth_store::physical_runtime::*;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(super) const MEMBER_BYTES: usize = 20 * 1024;
pub(super) const PAYLOAD: &[u8] = b"actual owned publication";
pub(super) fn name() -> AnchoredRecordName {
    AnchoredRecordName::new("physical.owned-checkpoint").unwrap()
}
pub(super) fn world(label: &str) -> PhysicalResidencyStoreWorld {
    PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
        label,
        std::num::NonZeroU64::new(64 * 1024).unwrap(),
    )
    .unwrap()
}
pub(super) fn publication(
    world: &PhysicalResidencyStoreWorld,
    members: usize,
) -> AnchoredRecordPublication {
    AnchoredRecordPublication::new(
        name(),
        0,
        PAYLOAD.to_vec(),
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .with_members(
        (0..members)
            .map(|i| {
                let bytes = vec![i as u8 + 1; MEMBER_BYTES];
                AnchoredRecordMember::New {
                    key: Sha256::digest(&bytes).into(),
                    bytes,
                }
            })
            .collect(),
    )
}
pub(super) fn view(
    memory: &SerialMemoryBudget,
    cancellation: &CancellationSource,
    deadline: Option<Instant>,
) -> ExecutionInterruptionView {
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), deadline);
    let mut retained = None;
    ExecutionRequest::serial(&request)
        .run_native_with_checkpoint(|context| {
            retained = Some(context.retain_interruption_view(None).unwrap());
        })
        .unwrap();
    retained.unwrap()
}
struct InputMemory {
    _original: ExecutionMemoryReservation,
}
struct InputProtection {
    _input: Arc<InputMemory>,
    release_panics: bool,
}
impl Drop for InputProtection {
    fn drop(&mut self) {
        if self.release_panics {
            panic!("domain input protection release panicked");
        }
    }
}
impl AnchoredRecordSelectionProtection for InputProtection {}
#[derive(Clone, Copy)]
pub(super) enum Delivery {
    Normal,
    PanicRegistration,
    RefuseRegistration,
    PanicTerminal,
    PanicRelease,
}
struct Destination(Delivery);
impl AnchoredRecordStepTerminalDestination for Destination {
    fn record_terminal(
        &self,
        _: &AnchoredRecordStepTerminal,
    ) -> Result<(), AnchoredRecordStepDeliveryFailure> {
        if matches!(self.0, Delivery::PanicTerminal) {
            panic!("domain terminal destination panicked");
        }
        Ok(())
    }
}
pub(super) struct Port {
    input: Arc<InputMemory>,
    destination: Arc<Destination>,
    registrations: Mutex<Vec<AnchoredRecordStepRegistration>>,
}
impl Port {
    pub(super) fn new(memory: &SerialMemoryBudget, members: usize, behavior: Delivery) -> Self {
        // The real ticket is issued BEFORE the corresponding input allocation.
        Self {
            input: Arc::new(InputMemory {
                _original: memory
                    .reserve((members * MEMBER_BYTES + PAYLOAD.len()) as u64)
                    .unwrap(),
            }),
            destination: Arc::new(Destination(behavior)),
            registrations: Mutex::new(Vec::new()),
        }
    }
    pub(super) fn registration(
        &self,
        index: usize,
    ) -> (PhysicalMutationIdentity, AnchoredRecordStepKind, usize) {
        let state = self.registrations.lock().unwrap();
        let r = &state[index];
        (
            r.mutation_identity(),
            r.kind(),
            r.completed_member_batches(),
        )
    }
    pub(super) fn take_registration(&self) -> AnchoredRecordStepRegistration {
        self.registrations.lock().unwrap().pop().unwrap()
    }
}
impl AnchoredRecordStepCustodyPort for Port {
    fn retain_step(
        &self,
        registration: AnchoredRecordStepRegistration,
    ) -> Result<AnchoredRecordStepRetention, AnchoredRecordStepCustodyRefusal> {
        self.registrations.lock().unwrap().push(registration);
        if matches!(self.destination.0, Delivery::RefuseRegistration) {
            return Err(AnchoredRecordStepCustodyRefusal::SelectionUnavailable);
        }
        if matches!(self.destination.0, Delivery::PanicRegistration) {
            panic!("domain registration panicked");
        }
        Ok(AnchoredRecordStepRetention::new(
            Box::new(InputProtection {
                _input: self.input.clone(),
                release_panics: matches!(self.destination.0, Delivery::PanicRelease),
            }),
            self.destination.clone(),
        ))
    }
}
pub(super) struct HeldGate(pub(super) PhysicalMutationPauseGate);
impl Drop for HeldGate {
    fn drop(&mut self) {
        self.0.release();
    }
}
pub(super) fn pause(
    serving: &ServingPhysicalRuntime,
    seam: PhysicalMutationCheckpoint,
) -> HeldGate {
    HeldGate(serving.pause_physical_mutation_at(seam))
}
pub(super) fn start(
    world: &PhysicalResidencyStoreWorld,
    members: usize,
    memory: &SerialMemoryBudget,
    cancellation: &CancellationSource,
    deadline: Option<Instant>,
) -> (
    AnchoredRecordPublicationOperation,
    AnchoredRecordPublicationRecovery,
) {
    match world
        .serving()
        .anchored_records()
        .unwrap()
        .start_publication(
            publication(world, members),
            view(memory, cancellation, deadline),
        ) {
        AnchoredRecordPublicationStart::Ready {
            operation,
            recovery,
        } => (operation, recovery),
        AnchoredRecordPublicationStart::Refused(cause) => {
            panic!("real admission refused: {cause:?}")
        }
        _ => panic!("fresh publication already selected"),
    }
}
pub(super) fn pending(
    operation: AnchoredRecordPublicationOperation,
    records: &AnchoredRecords<'_>,
    port: &Port,
) -> AnchoredRecordPublicationOperation {
    match operation.step(records, port) {
        AnchoredRecordPublicationStep::Pending(same) => same,
        AnchoredRecordPublicationStep::Failed(f) => panic!("held step refused: {:?}", f.cause()),
        _ => panic!("held step must remain pending"),
    }
}
pub(super) fn next(
    mut operation: AnchoredRecordPublicationOperation,
    records: &AnchoredRecords<'_>,
    port: &Port,
) -> AnchoredRecordPublicationStep {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match operation.step(records, port) {
            AnchoredRecordPublicationStep::Pending(same) => operation = same,
            terminal => return terminal,
        }
        assert!(Instant::now() < deadline, "current step did not settle");
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(super) fn settle(
    mut stopped: AnchoredRecordStoppedPublication,
    records: &AnchoredRecords<'_>,
) -> AnchoredRecordStoppedPoll {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match stopped.settle(records) {
            AnchoredRecordStoppedPoll::Pending(same) => stopped = same,
            terminal => return terminal,
        }
        assert!(
            Instant::now() < deadline,
            "stopped actual step did not settle"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(super) fn wait_terminal(
    registration: &AnchoredRecordStepRegistration,
) -> Arc<AnchoredRecordStepTerminal> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(actual) = registration.terminal() {
            return actual;
        }
        assert!(
            Instant::now() < deadline,
            "retained identity lost terminal evidence"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
