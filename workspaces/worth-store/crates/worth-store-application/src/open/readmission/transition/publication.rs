//! The Home retains the original intent operation independently of caller results.
use super::{
    intent, ApplicationHomeTransitionIntentDenial as Denial, HomeProgramTransitionIntent,
    ReservedApplicationHomeTransitionPlan,
};
use crate::frame_encoding::EncodedHomeFrame;
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionResourceLease};
use worth_store::physical_runtime::*;

mod progression;

#[derive(Debug)]
pub enum ApplicationHomeTransitionIntentOutcome {
    Pending,
    Committed,
    Refused(Arc<ApplicationHomeTransitionStartRefused>),
    Unresolved,
    SourceMismatch,
}
#[derive(Debug)]
pub struct ApplicationHomeTransitionStartRefused {
    cause: OrderedJournalOperationFailure,
    _funding: Option<ExecutionMemoryReservation>,
}
impl ApplicationHomeTransitionStartRefused {
    pub fn reason(&self) -> &OrderedJournalOperationFailure {
        &self.cause
    }
}

pub(super) struct HomeIntentPublication {
    pub(super) intent: HomeProgramTransitionIntent,
    pub(super) reserved: ReservedApplicationHomeTransitionPlan,
    pub(super) expected: OrderedJournalPublicationSourceDescription,
    pub(super) physical: Mutex<PhysicalIntent>,
    driver: AtomicBool,
    refusal_funding: Mutex<Option<ExecutionMemoryReservation>>,
    _funding: Option<ExecutionMemoryReservation>,
}
pub(super) struct OrderedJournalPublicationSourceDescription {
    runtime: RuntimeIdentity,
    position: u64,
    digest: [u8; 32],
    bytes: u64,
}
pub(super) struct PhysicalIntent {
    payload: Option<OrderedJournalPublication>,
    interruption: Option<worth_execution::ExecutionInterruptionView>,
    recovery: Option<OrderedJournalPublicationRecovery>,
    endpoint: Option<Endpoint>,
    invalid: bool,
}
enum Endpoint {
    Ready(OrderedJournalPublicationOperation),
    Stopped(OrderedJournalStoppedPublication),
    Committed {
        _committed: OrderedJournalCommitted,
    },
    Terminal {
        _observation: OrderedJournalPublicationObservation,
    },
    Refused(Arc<ApplicationHomeTransitionStartRefused>),
    InCaller,
}
impl HomeIntentPublication {
    pub(super) fn prepare(
        serving: &ServingPhysicalRuntime,
        reserved: ReservedApplicationHomeTransitionPlan,
        predecessor: &worth_foundational::facade::ApplicationHomeBinding,
        successor: &worth_foundational::facade::ApplicationHomeBinding,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let EncodedHomeFrame { bytes, memory } =
            intent::encode(&reserved, predecessor, successor, request)?;
        let capture = |lease: Option<&ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|context| context.retain_interruption_view(lease))
                .map_err(Denial::Execution)?
                .0
                .map_err(Denial::Interruption)
        };
        let interruption = if request.is_leased() {
            request.in_scope(capture).map_err(Denial::Execution)??
        } else {
            capture(None)?
        };
        let position = reserved.description().identity().intent_position().get();
        let intent = intent::decode(
            &bytes,
            reserved.description().identity().home(),
            position,
            Some(&reserved.bodies),
            request,
        )?;
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Denial::SizeLimit)?;
        let (failure_layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                ApplicationHomeTransitionStartRefused,
            >())
            .map_err(|_| Denial::SizeLimit)?;
        let refusal_funding = request
            .reserve_retained_memory(failure_layout.pad_to_align().size() as u64)
            .map_err(Denial::Execution)?;
        let funding = request
            .reserve_retained_memory(layout.pad_to_align().size() as u64)
            .map_err(Denial::Execution)?;
        let milliseconds =
            crate::deadline::milliseconds(request, 120_000).map_err(Denial::Deadline)?;
        let expected = OrderedJournalPublicationSourceDescription {
            runtime: serving.runtime_identity(),
            position,
            digest: Sha256::digest(&bytes).into(),
            bytes: bytes.len() as u64,
        };
        let mut publication = OrderedJournalPublication::new(
            position,
            bytes,
            crate::open::physical_admission::placement(),
            PhysicalMutationDeadline::after_milliseconds(milliseconds)
                .expect("admitted positive intent deadline"),
        );
        if let Some(memory) = memory {
            publication = publication.with_input_memory(memory);
        }
        Ok(Arc::new(Self {
            intent,
            reserved,
            expected,
            physical: Mutex::new(PhysicalIntent {
                payload: Some(publication),
                interruption: Some(interruption),
                recovery: None,
                endpoint: None,
                invalid: false,
            }),
            driver: AtomicBool::new(false),
            refusal_funding: Mutex::new(refusal_funding),
            _funding: funding,
        }))
    }
    pub(super) fn state(&self) -> std::sync::MutexGuard<'_, PhysicalIntent> {
        self.physical
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
    }
    pub(super) fn advance(
        &self,
        serving: &ServingPhysicalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeTransitionIntentOutcome, Denial> {
        if self
            .driver
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(Denial::Busy);
        }
        let _driver = Driver(self);
        self.resume();
        if let Some(outcome) = self.observation() {
            return Ok(outcome);
        }
        let journal = serving
            .ordered_journal(
                OrderedJournalName::new("home.transitions").expect("closed Home journal"),
            )
            .map_err(Denial::Journal)?;
        let run = |_: Option<&ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|_| {
                    let (payload, view) = {
                        let mut state = self.state();
                        (state.payload.take(), state.interruption.take())
                    };
                    if let Some(payload) = payload {
                        match journal.start_publication(
                            payload,
                            view.expect("unsubmitted payload retained its original controls"),
                        ) {
                            OrderedJournalPublicationStart::Ready {
                                operation,
                                recovery,
                            } => self.install(operation, recovery),
                            OrderedJournalPublicationStart::AlreadyCommitted(committed) => {
                                self.committed(committed)
                            }
                            OrderedJournalPublicationStart::Refused(cause) => {
                                let funding = self
                                    .refusal_funding
                                    .lock()
                                    .unwrap_or_else(|cause| cause.into_inner())
                                    .take();
                                self.state().endpoint = Some(Endpoint::Refused(Arc::new(
                                    ApplicationHomeTransitionStartRefused {
                                        cause,
                                        _funding: funding,
                                    },
                                )));
                            }
                        }
                    }
                    if self.state().invalid {
                        self.stop();
                    } else {
                        self.step(&journal);
                    }
                })
                .map(|_| ())
        };
        let outcome = if request.is_leased() {
            request.in_scope(run).and_then(|outcome| outcome)
        } else {
            run(None)
        };
        // Recover actual completion outside the disposable invoking context.
        self.resume();
        if let Some(observed) = self.observation() {
            return Ok(observed);
        }
        if let Err(cause) = outcome {
            self.stop();
            if let Some(observed) = self.observation() {
                return Ok(observed);
            }
            return Err(Denial::Execution(cause));
        }
        Ok(self
            .observation()
            .unwrap_or(ApplicationHomeTransitionIntentOutcome::Pending))
    }
    pub(super) fn matches(&self, source: OrderedJournalPublicationSource) -> bool {
        source.runtime_identity() == self.expected.runtime
            && source.journal().as_str() == "home.transitions"
            && source.position() == self.expected.position
            && source.payload_sha256() == self.expected.digest
            && source.payload_bytes() == self.expected.bytes
    }
    pub(super) fn committed(&self, committed: OrderedJournalCommitted) {
        let valid = self.matches(committed.source());
        let mut state = self.state();
        state.invalid |= !valid;
        state.endpoint = Some(Endpoint::Committed {
            _committed: committed,
        });
    }
    pub(super) fn install(
        &self,
        operation: OrderedJournalPublicationOperation,
        recovery: OrderedJournalPublicationRecovery,
    ) {
        let valid =
            self.matches(operation.observe().source()) && self.matches(recovery.observe().source());
        let mut state = self.state();
        state.invalid |= !valid;
        state.recovery = Some(recovery);
        state.endpoint = Some(Endpoint::Ready(operation));
    }
    pub(super) fn observation(&self) -> Option<ApplicationHomeTransitionIntentOutcome> {
        let state = self.state();
        if state.invalid {
            return Some(ApplicationHomeTransitionIntentOutcome::SourceMismatch);
        }
        match state.endpoint.as_ref()? {
            Endpoint::Committed { .. } => Some(ApplicationHomeTransitionIntentOutcome::Committed),
            Endpoint::Refused(cause) => Some(ApplicationHomeTransitionIntentOutcome::Refused(
                cause.clone(),
            )),
            Endpoint::Terminal { .. } => Some(ApplicationHomeTransitionIntentOutcome::Unresolved),
            Endpoint::Ready(_) | Endpoint::Stopped(_) | Endpoint::InCaller => None,
        }
    }
}
struct Driver<'a>(&'a HomeIntentPublication);
impl Drop for Driver<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.stop();
        }
        self.0.driver.store(false, Ordering::Release);
    }
}
