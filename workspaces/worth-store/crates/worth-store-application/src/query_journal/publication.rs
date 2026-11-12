//! The original physical journal endpoint, retained across invoking requests.
use super::*;
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionInterruptionView, ExecutionResourceLease};
mod execution;
mod terminal;

pub(super) enum QueryJournalNativeProgress {
    Pending,
    Committed { position: u64, digest: [u8; 32] },
    ProvenNoEffect { position: u64, digest: [u8; 32] },
    Refused(QueryJournalDenial),
    Unresolved,
}
pub(super) struct QueryJournalPublication {
    journal: &'static str,
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
impl QueryJournalPublication {
    pub(in crate::query_journal) fn stop_before_start_for_test(&self) {
        self.stop();
    }
    pub(super) fn prepare(
        serving: &ServingPhysicalRuntime,
        journal: &'static str,
        position: u64,
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, QueryJournalDenial> {
        let capture = |lease: Option<&ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|context| context.retain_interruption_view(lease))
                .map_err(QueryJournalDenial::Execution)?
                .0
                .map_err(storage)
        };
        let view = if request.is_leased() {
            request
                .in_scope(capture)
                .map_err(QueryJournalDenial::Execution)??
        } else {
            capture(None)?
        };
        let milliseconds = crate::deadline::milliseconds(request, 120_000).map_err(storage)?;
        let payload = OrderedJournalPublication::new(
            position,
            bytes.to_vec(),
            crate::open::physical_admission::placement(),
            PhysicalMutationDeadline::after_milliseconds(milliseconds)
                .ok_or(QueryJournalDenial::Unavailable)?,
        );
        Ok(Self {
            journal,
            position,
            digest: Sha256::digest(bytes).into(),
            bytes: bytes.len() as u64,
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

    pub(super) fn matches_payload(&self, position: u64, bytes: &[u8]) -> bool {
        self.position == position
            && self.digest == <[u8; 32]>::from(Sha256::digest(bytes))
            && self.bytes == bytes.len() as u64
    }
    pub(super) fn matches_subject(&self, position: u64, digest: [u8; 32]) -> bool {
        self.position == position && self.digest == digest
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
            Some(QueryJournalNativeProgress::ProvenNoEffect { .. }) => true,
            Some(
                QueryJournalNativeProgress::Pending
                | QueryJournalNativeProgress::Committed { .. }
                | QueryJournalNativeProgress::Refused(_)
                | QueryJournalNativeProgress::Unresolved,
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
            && source.journal().as_str() == self.journal
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
