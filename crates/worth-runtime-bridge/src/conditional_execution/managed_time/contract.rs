pub(super) const MAXIMUM_IDENTITY_BYTES: usize = 512;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use worth_signal::facade::{TemporalWakeId, WakeOrdinal};

#[derive(Debug)]
pub(super) struct BridgeManagedClockLease {
    live: AtomicBool,
    _reservation: super::super::retention::BridgeRetentionReservation,
}

impl BridgeManagedClockLease {
    pub(super) fn issue(reservation: super::super::retention::BridgeRetentionReservation) -> Self {
        Self {
            live: AtomicBool::new(true),
            _reservation: reservation,
        }
    }

    pub(super) fn revoke(&self) {
        self.live.store(false, Ordering::Release);
    }

    fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }
}

/// Exact Bridge-owned clock binding. It is move-only and cannot schedule work.
pub struct BridgeManagedClockBinding {
    pub(in crate::conditional_execution) bridge_runtime_key: u64,
    pub(in crate::conditional_execution) binding_identity: Arc<str>,
    pub(super) source_identity: Arc<str>,
    pub(super) timeline_identity: Arc<str>,
    pub(super) lease: Arc<BridgeManagedClockLease>,
}

impl BridgeManagedClockBinding {
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    pub fn timeline_identity(&self) -> &str {
        &self.timeline_identity
    }

    pub fn is_live(&self) -> bool {
        self.lease.is_live()
    }
}

pub struct BridgeManagedClockInstallationParts<'a> {
    pub lowering: &'a std::sync::Arc<super::super::BridgeInstalledConditionalLowering>,
    pub binding_identity: Arc<str>,
    pub source_identity: Arc<str>,
    pub timeline_identity: Arc<str>,
    pub maximum_active_intents: usize,
    pub maximum_due_wakes_per_observation: usize,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BridgeManagedTemporalIntentIdentity(Arc<str>);

impl BridgeManagedTemporalIntentIdentity {
    pub fn declare(identity: impl Into<Arc<str>>) -> Result<Self, BridgeManagedTemporalDenial> {
        let identity = identity.into();
        validate_identity(&identity, "temporal intent")?;
        Ok(Self(identity))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedTemporalIntentLifecycle {
    Active,
    Cancelled,
    Completed,
}

pub struct BridgeManagedTemporalIntentReconciliationParts<'a> {
    pub binding: &'a BridgeManagedClockBinding,
    pub identity: BridgeManagedTemporalIntentIdentity,
    pub revision: u64,
    pub due_coordinate: u64,
    pub idempotency_identity: Arc<str>,
    pub source_record_identity: crate::relational_identity::RelationalBridgeRecordIdentityParts,
    pub lifecycle: BridgeManagedTemporalIntentLifecycle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedTemporalIntentReconciliation {
    Installed,
    Duplicate,
    Superseded,
    Retired,
    TerminalNoop,
    Stale,
}

pub struct BridgeManagedClockObservationParts<'a> {
    pub binding: &'a BridgeManagedClockBinding,
    pub source_identity: &'a str,
    pub timeline_identity: &'a str,
    pub sequence: u64,
    pub observed_coordinate: u64,
}

/// Move-only Bridge evidence joining one exact Signal wake to the durable
/// temporal-intent association retained by Bridge.
pub struct BridgeManagedDueWake {
    pub(super) binding_identity: Arc<str>,
    pub(super) intent_identity: BridgeManagedTemporalIntentIdentity,
    pub(super) revision: u64,
    pub(super) idempotency_identity: Arc<str>,
    pub(super) source_record_identity:
        crate::relational_identity::RelationalBridgeRecordIdentityParts,
    pub(super) due_coordinate: u64,
    pub(super) ready_coordinate: u64,
    pub(in crate::conditional_execution) signal_wake_id: TemporalWakeId,
    pub(super) scheduled_ordinal: WakeOrdinal,
    pub(super) ready_ordinal: WakeOrdinal,
    pub(in crate::conditional_execution) observation_baselines:
        Arc<super::super::observation_retention::BridgeObservationBaselines>,
    pub(super) _reservation: Arc<super::super::retention::BridgeRetentionReservation>,
}

impl BridgeManagedDueWake {
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn intent_identity(&self) -> &BridgeManagedTemporalIntentIdentity {
        &self.intent_identity
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn idempotency_identity(&self) -> &str {
        &self.idempotency_identity
    }

    pub fn source_record_identity(
        &self,
    ) -> crate::relational_identity::RelationalBridgeRecordIdentityParts {
        self.source_record_identity
    }

    pub fn due_coordinate(&self) -> u64 {
        self.due_coordinate
    }

    pub fn ready_coordinate(&self) -> u64 {
        self.ready_coordinate
    }

    pub fn signal_scheduled_ordinal(&self) -> u64 {
        self.scheduled_ordinal.get()
    }

    pub fn signal_ready_ordinal(&self) -> u64 {
        self.ready_ordinal.get()
    }
}

pub struct BridgeManagedDueWakeBatch {
    pub(super) wakes: super::BridgeManagedDueWakeBuffer,
    pub(super) due_work_remaining: bool,
    pub(super) frontier_width_before: u64,
    pub(super) frontier_width_after: u64,
}

impl BridgeManagedDueWakeBatch {
    pub fn wakes(&self) -> &[BridgeManagedDueWake] {
        &self.wakes
    }

    pub fn into_wakes(self) -> super::BridgeManagedDueWakeBuffer {
        self.wakes
    }

    pub fn due_work_remaining(&self) -> bool {
        self.due_work_remaining
    }

    pub fn frontier_width_before(&self) -> u64 {
        self.frontier_width_before
    }

    pub fn frontier_width_after(&self) -> u64 {
        self.frontier_width_after
    }
}

pub struct BridgeManagedClockAcceptedObservation {
    pub(super) sequence: u64,
    pub(super) observed_coordinate: u64,
    pub(super) signal_advance_ordinal: Option<u64>,
    pub(super) due: BridgeManagedDueWakeBatch,
}

impl BridgeManagedClockAcceptedObservation {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn observed_coordinate(&self) -> u64 {
        self.observed_coordinate
    }

    pub fn signal_advance_ordinal(&self) -> Option<u64> {
        self.signal_advance_ordinal
    }

    pub fn due(&self) -> &BridgeManagedDueWakeBatch {
        &self.due
    }

    pub fn into_due(self) -> BridgeManagedDueWakeBatch {
        self.due
    }
}

pub enum BridgeManagedClockObservationOutcome {
    Accepted(BridgeManagedClockAcceptedObservation),
    Duplicate(BridgeManagedClockAcceptedObservation),
    Stale,
    Reordered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedTemporalDenialKind {
    InvalidContract,
    DuplicateClockBinding,
    ForeignClockBinding,
    ClosedClockBinding,
    ForeignClockSource,
    ForeignClockTimeline,
    IntentCapacityExhausted,
    RetentionCapacityExhausted,
    RetentionQuarantined,
    LaneQuarantined,
    IntentRevisionConflict,
    SignalTemporalFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeManagedTemporalDenial {
    kind: BridgeManagedTemporalDenialKind,
    detail: String,
}

impl BridgeManagedTemporalDenial {
    pub(in crate::conditional_execution) fn new(
        kind: BridgeManagedTemporalDenialKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> BridgeManagedTemporalDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub struct BridgeManagedClockClosure {
    active_intents: usize,
    scheduled_wakes: usize,
    ready_wakes: usize,
}

impl BridgeManagedClockClosure {
    pub(super) fn new(active_intents: usize, scheduled_wakes: usize, ready_wakes: usize) -> Self {
        Self {
            active_intents,
            scheduled_wakes,
            ready_wakes,
        }
    }

    pub fn active_intents(&self) -> usize {
        self.active_intents
    }

    pub fn scheduled_wakes(&self) -> usize {
        self.scheduled_wakes
    }

    pub fn ready_wakes(&self) -> usize {
        self.ready_wakes
    }
}

pub(super) fn validate_identity(
    identity: &str,
    role: &str,
) -> Result<(), BridgeManagedTemporalDenial> {
    if identity.is_empty()
        || identity.len() > MAXIMUM_IDENTITY_BYTES
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(BridgeManagedTemporalDenial::new(
            BridgeManagedTemporalDenialKind::InvalidContract,
            format!("invalid {role} identity"),
        ))
    } else {
        Ok(())
    }
}
