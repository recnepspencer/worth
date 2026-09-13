use super::UiHostObservationBatch;
use std::collections::{BTreeSet, VecDeque};
use std::sync::Mutex;

mod pointer_motion;

pub const UI_HOST_OBSERVATION_ACTIVE_SESSION_LIMIT: usize = 16;
pub const UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT: usize = 16;
pub const UI_HOST_OBSERVATION_DRAIN_REPORT_LIMIT: usize = 256;
pub const UI_HOST_OBSERVATION_DRAIN_BYTE_LIMIT: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationDrainDenial {
    BatchCapacityExceeded,
    ReportCapacityExceeded,
    ByteCapacityExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationRetentionDenial {
    InactiveSession,
    Capacity(UiHostObservationDrainDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationSessionRegistrationDenial {
    ActiveSessionCapacityExceeded,
}

/// Adapter-issued, mechanically bounded raw observation transfer.
///
/// Construction checks storage represented by the actual reports rather than
/// trusting the batch's unvalidated canonical byte and report claims.
pub struct UiHostObservationDrain {
    batches: Box<[UiHostObservationBatch]>,
}

/// Standard adapter-owned bounded retention for raw host reports.
///
/// The adapter registers a session only when runtime opens concrete host
/// authority. Retention then accepts reports only while that session remains
/// active. Runtime and application callers can request a drain only through
/// the adapter contract and never receive a cloneable queue.
pub struct UiHostObservationRetention {
    state: Mutex<UiHostObservationRetentionState>,
}

struct UiHostObservationRetentionState {
    active_sessions: BTreeSet<u64>,
    batches: VecDeque<UiHostObservationBatch>,
    reports: usize,
    bytes: usize,
}

impl UiHostObservationDrain {
    pub fn bounded(
        batches: Vec<UiHostObservationBatch>,
    ) -> Result<Self, UiHostObservationDrainDenial> {
        if batches.len() > UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT {
            return Err(UiHostObservationDrainDenial::BatchCapacityExceeded);
        }
        let (reports, bytes) = measure_batches(&batches)?;
        if reports > UI_HOST_OBSERVATION_DRAIN_REPORT_LIMIT {
            return Err(UiHostObservationDrainDenial::ReportCapacityExceeded);
        }
        if bytes > UI_HOST_OBSERVATION_DRAIN_BYTE_LIMIT {
            return Err(UiHostObservationDrainDenial::ByteCapacityExceeded);
        }
        Ok(Self {
            batches: batches.into_boxed_slice(),
        })
    }

    pub fn empty() -> Self {
        Self {
            batches: Box::new([]),
        }
    }

    pub fn into_batches(self) -> Box<[UiHostObservationBatch]> {
        self.batches
    }
}

impl Default for UiHostObservationRetention {
    fn default() -> Self {
        Self {
            state: Mutex::new(UiHostObservationRetentionState {
                active_sessions: BTreeSet::new(),
                batches: VecDeque::new(),
                reports: 0,
                bytes: 0,
            }),
        }
    }
}

impl UiHostObservationRetention {
    pub fn register_session(
        &self,
        host_session_identity: u64,
    ) -> Result<(), UiHostObservationSessionRegistrationDenial> {
        let mut state = self
            .state
            .lock()
            .expect("host observation retention poisoned");
        register_active_session(&mut state, host_session_identity)
    }

    pub fn retain(
        &self,
        batch: UiHostObservationBatch,
    ) -> Result<(), UiHostObservationRetentionDenial> {
        self.retain_with_pointer_coalescing(batch, false)
    }

    /// Retains ordered input, replacing only an immediately preceding compatible
    /// pointer motion and explicitly reporting its replaced sequence range.
    pub fn retain_latest_pointer_motion(
        &self,
        batch: UiHostObservationBatch,
    ) -> Result<(), UiHostObservationRetentionDenial> {
        self.retain_with_pointer_coalescing(batch, true)
    }

    fn retain_with_pointer_coalescing(
        &self,
        batch: UiHostObservationBatch,
        coalesce_pointer: bool,
    ) -> Result<(), UiHostObservationRetentionDenial> {
        let (reports, bytes) = measure_batches(std::slice::from_ref(&batch))
            .map_err(UiHostObservationRetentionDenial::Capacity)?;
        let mut state = self
            .state
            .lock()
            .expect("host observation retention poisoned");
        let host_session = batch.canonical_core().host_session();
        if !state.active_sessions.contains(&host_session) {
            return Err(UiHostObservationRetentionDenial::InactiveSession);
        }
        if coalesce_pointer {
            if let Some(merged) = state
                .batches
                .back()
                .and_then(|previous| pointer_motion::merge(previous, &batch))
            {
                // Compatible motion has identical payload shape and attribution.
                // Replacing its position/time therefore changes no retained size.
                *state
                    .batches
                    .back_mut()
                    .expect("merged retained predecessor") = merged;
                return Ok(());
            }
        }
        if state.batches.len() == UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT {
            return Err(UiHostObservationRetentionDenial::Capacity(
                UiHostObservationDrainDenial::BatchCapacityExceeded,
            ));
        }
        let next_reports = state.reports.checked_add(reports).ok_or(
            UiHostObservationRetentionDenial::Capacity(
                UiHostObservationDrainDenial::ReportCapacityExceeded,
            ),
        )?;
        let next_bytes =
            state
                .bytes
                .checked_add(bytes)
                .ok_or(UiHostObservationRetentionDenial::Capacity(
                    UiHostObservationDrainDenial::ByteCapacityExceeded,
                ))?;
        if next_reports > UI_HOST_OBSERVATION_DRAIN_REPORT_LIMIT {
            return Err(UiHostObservationRetentionDenial::Capacity(
                UiHostObservationDrainDenial::ReportCapacityExceeded,
            ));
        }
        if next_bytes > UI_HOST_OBSERVATION_DRAIN_BYTE_LIMIT {
            return Err(UiHostObservationRetentionDenial::Capacity(
                UiHostObservationDrainDenial::ByteCapacityExceeded,
            ));
        }
        state.batches.push_back(batch);
        state.reports = next_reports;
        state.bytes = next_bytes;
        Ok(())
    }

    pub fn is_session_active(&self, host_session_identity: u64) -> bool {
        self.state
            .lock()
            .expect("host observation retention poisoned")
            .active_sessions
            .contains(&host_session_identity)
    }

    pub fn pending_batch_count(&self) -> usize {
        self.state
            .lock()
            .expect("host observation retention poisoned")
            .batches
            .len()
    }

    pub fn pending_batch_count_for(&self, host_session_identity: u64) -> usize {
        self.state
            .lock()
            .expect("host observation retention poisoned")
            .batches
            .iter()
            .filter(|batch| batch.canonical_core().host_session() == host_session_identity)
            .count()
    }

    pub fn drain(&self, host_session_identity: u64) -> UiHostObservationDrain {
        let mut state = self
            .state
            .lock()
            .expect("host observation retention poisoned");
        let batches = take_session_batches(&mut state, host_session_identity);
        UiHostObservationDrain {
            batches: batches.into_boxed_slice(),
        }
    }

    pub fn release_session(&self, host_session_identity: u64) {
        let mut state = self
            .state
            .lock()
            .expect("host observation retention poisoned");
        take_session_batches(&mut state, host_session_identity);
        state.active_sessions.remove(&host_session_identity);
    }
}

fn register_active_session(
    state: &mut UiHostObservationRetentionState,
    host_session_identity: u64,
) -> Result<(), UiHostObservationSessionRegistrationDenial> {
    if state.active_sessions.contains(&host_session_identity) {
        return Ok(());
    }
    if state.active_sessions.len() == UI_HOST_OBSERVATION_ACTIVE_SESSION_LIMIT {
        return Err(UiHostObservationSessionRegistrationDenial::ActiveSessionCapacityExceeded);
    }
    state.active_sessions.insert(host_session_identity);
    Ok(())
}

fn take_session_batches(
    state: &mut UiHostObservationRetentionState,
    host_session_identity: u64,
) -> Vec<UiHostObservationBatch> {
    let mut retained = VecDeque::with_capacity(state.batches.len());
    let mut drained = Vec::new();
    while let Some(batch) = state.batches.pop_front() {
        if batch.canonical_core().host_session() == host_session_identity {
            let (reports, bytes) = measure_batches(std::slice::from_ref(&batch))
                .expect("retained host observation remains mechanically measurable");
            state.reports -= reports;
            state.bytes -= bytes;
            drained.push(batch);
        } else {
            retained.push_back(batch);
        }
    }
    state.batches = retained;
    drained
}

fn measure_batches(
    batches: &[UiHostObservationBatch],
) -> Result<(usize, usize), UiHostObservationDrainDenial> {
    let mut reports = 0usize;
    let mut bytes = 0usize;
    for batch in batches {
        reports = reports
            .checked_add(batch.reports().len())
            .ok_or(UiHostObservationDrainDenial::ReportCapacityExceeded)?;
        bytes = batch.reports().iter().try_fold(bytes, |total, report| {
            total
                .checked_add(report.encoded_len())
                .ok_or(UiHostObservationDrainDenial::ByteCapacityExceeded)
        })?;
    }
    Ok((reports, bytes))
}

#[cfg(test)]
#[path = "drain/tests.rs"]
mod tests;
