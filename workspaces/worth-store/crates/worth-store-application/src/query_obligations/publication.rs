//! The original physical journal endpoint, retained across invoking requests.
use super::*;
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionInterruptionView, ExecutionResourceLease};
mod execution;
mod terminal;

pub(super) enum QueryObligationNativeProgress {
    Pending,
    Committed { position: u64, digest: [u8; 32] },
    ProvenNoEffect { position: u64, digest: [u8; 32] },
    Refused(QueryProgramSupportDenial),
    Unresolved,
}
pub(super) struct QueryObligationPublication {
    position: u64,
    digest: [u8; 32],
    bytes: u64,
    runtime: RuntimeIdentity,
    physical: Mutex<PhysicalPublication>,
}
struct PhysicalPublication {
    payload: Option<OrderedJournalPublication>,
    view: Option<ExecutionInterruptionView>,
    endpoint: Endpoint,
    recovery: Option<OrderedJournalPublicationRecovery>,
    invalid: bool,
}
enum Endpoint {
    Unstarted,
    InCaller,
    Ready(OrderedJournalPublicationOperation),
    Stopped(OrderedJournalStoppedPublication),
    Committed(OrderedJournalCommitted),
    Refused(OrderedJournalOperationFailure),
    Terminal(OrderedJournalPublicationObservation),
    NoEffect(OrderedJournalPublicationObservation),
    NoEffectBeforeStart,
}
impl QueryObligationPublication {
    pub(super) fn prepare(
        serving: &ServingPhysicalRuntime,
        frame: &QueryProgramSupportFrame,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, QueryProgramSupportDenial> {
        let capture = |lease: Option<&ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|context| context.retain_interruption_view(lease))
                .map_err(QueryProgramSupportDenial::Execution)?
                .0
                .map_err(storage)
        };
        let view = if request.is_leased() {
            request
                .in_scope(capture)
                .map_err(QueryProgramSupportDenial::Execution)??
        } else {
            capture(None)?
        };
        let milliseconds = crate::deadline::milliseconds(request, 120_000).map_err(storage)?;
        let payload = OrderedJournalPublication::new(
            frame.position(),
            frame.bytes().to_vec(),
            crate::open::physical_admission::placement(),
            PhysicalMutationDeadline::after_milliseconds(milliseconds)
                .ok_or(QueryProgramSupportDenial::SupportUnavailable)?,
        );
        Ok(Self {
            position: frame.position(),
            digest: Sha256::digest(frame.bytes()).into(),
            bytes: frame.bytes().len() as u64,
            runtime: serving.runtime_identity(),
            physical: Mutex::new(PhysicalPublication {
                payload: Some(payload),
                view: Some(view),
                endpoint: Endpoint::Unstarted,
                recovery: None,
                invalid: false,
            }),
        })
    }

    pub(super) fn matches_frame(&self, frame: &QueryProgramSupportFrame) -> bool {
        self.position == frame.position()
            && self.digest == frame.payload_digest()
            && self.bytes == frame.bytes().len() as u64
    }
    pub(super) fn matches_subject(
        &self,
        subject: &worth_proof::HomeQueryObligationSubject,
    ) -> bool {
        self.position == subject.position() && self.digest == subject.payload_digest()
    }
    pub(super) fn committed(&self) -> bool {
        let state = self.state();
        !state.invalid
            && match &state.endpoint {
                Endpoint::Committed(_) => true,
                Endpoint::Unstarted
                | Endpoint::InCaller
                | Endpoint::Ready(_)
                | Endpoint::Stopped(_)
                | Endpoint::Refused(_)
                | Endpoint::Terminal(_)
                | Endpoint::NoEffect(_)
                | Endpoint::NoEffectBeforeStart => false,
            }
    }
    pub(super) fn no_effect(&self) -> bool {
        match self.outcome() {
            Some(QueryObligationNativeProgress::ProvenNoEffect { .. }) => true,
            Some(
                QueryObligationNativeProgress::Pending
                | QueryObligationNativeProgress::Committed { .. }
                | QueryObligationNativeProgress::Refused(_)
                | QueryObligationNativeProgress::Unresolved,
            )
            | None => false,
        }
    }
    fn state(&self) -> std::sync::MutexGuard<'_, PhysicalPublication> {
        self.physical
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn matches(&self, source: OrderedJournalPublicationSource) -> bool {
        source.runtime_identity() == self.runtime
            && source.journal().as_str() == "query.obligations"
            && source.position() == self.position
            && source.payload_sha256() == self.digest
            && source.payload_bytes() == self.bytes
    }
    fn install(
        &self,
        operation: OrderedJournalPublicationOperation,
        recovery: OrderedJournalPublicationRecovery,
    ) {
        let valid =
            self.matches(operation.observe().source()) && self.matches(recovery.observe().source());
        let mut state = self.state();
        state.invalid |= !valid;
        state.recovery = Some(recovery);
        state.endpoint = Endpoint::Ready(operation);
    }
    fn accept(&self, committed: OrderedJournalCommitted) {
        let valid = self.matches(committed.source());
        let mut state = self.state();
        state.invalid |= !valid;
        state.endpoint = Endpoint::Committed(committed);
        state.recovery = None;
    }
    fn resume(&self) {
        let recovery = {
            let mut state = self.state();
            match state.endpoint {
                Endpoint::InCaller | Endpoint::Terminal(_) => state.recovery.take(),
                Endpoint::Unstarted
                | Endpoint::Ready(_)
                | Endpoint::Stopped(_)
                | Endpoint::Committed(_)
                | Endpoint::Refused(_)
                | Endpoint::NoEffect(_)
                | Endpoint::NoEffectBeforeStart => return,
            }
        };
        let Some(recovery) = recovery else {
            return;
        };
        match recovery.resume() {
            OrderedJournalPublicationResume::Committed(committed) => self.accept(committed),
            OrderedJournalPublicationResume::Ready {
                operation,
                recovery,
            } => self.install(operation, recovery),
            OrderedJournalPublicationResume::Stopped {
                operation,
                recovery,
            } => {
                let mut state = self.state();
                state.recovery = Some(recovery);
                state.endpoint = Endpoint::Stopped(operation);
            }
            OrderedJournalPublicationResume::Busy(recovery) => {
                self.state().recovery = Some(recovery);
            }
            OrderedJournalPublicationResume::Terminal(observed) => {
                let valid = self.matches(observed.source());
                let no_effect = self.proven_no_effect(&observed);
                let mut state = self.state();
                state.invalid |= !valid;
                state.endpoint = if no_effect {
                    Endpoint::NoEffect(observed)
                } else {
                    Endpoint::Terminal(observed)
                };
            }
        }
    }
    fn stop(&self) {
        let mut state = self.state();
        state.recovery = state
            .recovery
            .take()
            .map(OrderedJournalPublicationRecovery::stop);
        let previous = std::mem::replace(&mut state.endpoint, Endpoint::InCaller);
        state.endpoint = match previous {
            Endpoint::Unstarted => Endpoint::NoEffectBeforeStart,
            Endpoint::Ready(operation) => Endpoint::Stopped(operation.stop()),
            same @ (Endpoint::InCaller
            | Endpoint::Stopped(_)
            | Endpoint::Committed(_)
            | Endpoint::Refused(_)
            | Endpoint::Terminal(_)
            | Endpoint::NoEffect(_)
            | Endpoint::NoEffectBeforeStart) => same,
        };
    }
}
