//! Types and NDJSON trace encoder for `specs/tla/store_m3/StoreDurabilityTrace.tla`.
use super::super::super::{BUILD_PRIMARY, BUILD_SECONDARY, BUILD_TERTIARY};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PendingWrite {
    None,
    JournalUnflushed,
    JournalFlushedPreManifest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LeaseMode {
    None,
    CommitLease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ForkPhase {
    Idle,
    Creating,
    ReopenRequired,
    ReopenBusy,
    Published,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecoveryStage {
    None,
    Staging,
    ManifestSwitch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DerivedState {
    Covered,
    DiscardedBlocked,
    DiscardedReopened,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BuildTag {
    B1,
    B2,
    B3,
}

impl BuildTag {
    pub(super) fn as_tla(self) -> &'static str {
        build_str(self)
    }

    pub(super) fn as_uuid(self) -> &'static str {
        match self {
            Self::B1 => BUILD_PRIMARY,
            Self::B2 => BUILD_SECONDARY,
            Self::B3 => BUILD_TERTIARY,
        }
    }

    pub(super) fn others(self) -> [Self; 2] {
        match self {
            Self::B1 => [Self::B2, Self::B3],
            Self::B2 => [Self::B1, Self::B3],
            Self::B3 => [Self::B1, Self::B2],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TlaModelState {
    pub durable_head: u64,
    pub checkpoint_epoch: u64,
    pub staged_checkpoint: u64,
    pub retained_floor: u64,
    pub pending_write: PendingWrite,
    pub pending_target: u64,
    pub active_lease: LeaseMode,
    pub fork_phase: ForkPhase,
    pub crashed: bool,
    pub recovery_stage: RecoveryStage,
    pub build_id: BuildTag,
    pub derived_state: DerivedState,
    pub recovered_head: u64,
    pub reopen_succeeded: bool,
}

impl Default for TlaModelState {
    fn default() -> Self {
        Self {
            durable_head: 0,
            checkpoint_epoch: 0,
            staged_checkpoint: 0,
            retained_floor: 0,
            pending_write: PendingWrite::None,
            pending_target: 0,
            active_lease: LeaseMode::None,
            fork_phase: ForkPhase::Idle,
            crashed: false,
            recovery_stage: RecoveryStage::None,
            build_id: BuildTag::B1,
            derived_state: DerivedState::Covered,
            recovered_head: 0,
            reopen_succeeded: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TlaAction {
    BeginCommit,
    FlushJournal,
    SealManifest,
    BeginCheckpoint { epoch: u64 },
    SealCheckpoint,
    RetireHistoryPrefix { new_floor: u64 },
    DiscardDerivedState,
    GracefulReopen { build_id: BuildTag },
    BeginForkCreate,
    ForkNeedsReopen,
    ForkReopenBusy,
    ForkBusyRetryClear,
    ReopenForkToPublished,
    PublishForkDirect,
    CrashKill,
    BeginRecover { build_id: BuildTag },
    AdvanceRecoverStage { resolved_head: u64 },
    FinishRecover,
}

pub(super) fn pending_str(p: PendingWrite) -> &'static str {
    match p {
        PendingWrite::None => "none",
        PendingWrite::JournalUnflushed => "journal_unflushed",
        PendingWrite::JournalFlushedPreManifest => "journal_flushed_pre_manifest",
    }
}

pub(super) fn lease_str(l: LeaseMode) -> &'static str {
    match l {
        LeaseMode::None => "none",
        LeaseMode::CommitLease => "commit_lease",
    }
}

pub(super) fn fork_str(f: ForkPhase) -> &'static str {
    match f {
        ForkPhase::Idle => "Idle",
        ForkPhase::Creating => "Creating",
        ForkPhase::ReopenRequired => "ReopenRequired",
        ForkPhase::ReopenBusy => "ReopenBusy",
        ForkPhase::Published => "Published",
    }
}

pub(super) fn recovery_str(r: RecoveryStage) -> &'static str {
    match r {
        RecoveryStage::None => "none",
        RecoveryStage::Staging => "staging",
        RecoveryStage::ManifestSwitch => "manifest_switch",
    }
}

pub(super) fn derived_str(d: DerivedState) -> &'static str {
    match d {
        DerivedState::Covered => "covered",
        DerivedState::DiscardedBlocked => "discarded_blocked",
        DerivedState::DiscardedReopened => "discarded_reopened",
    }
}

pub(super) fn build_str(b: BuildTag) -> &'static str {
    match b {
        BuildTag::B1 => "b1",
        BuildTag::B2 => "b2",
        BuildTag::B3 => "b3",
    }
}

pub(super) fn encode_trace_row(action: TlaAction, s: TlaModelState) -> Value {
    let mut obj = json!({
        "action": match action {
            TlaAction::BeginCommit => "BeginCommit",
            TlaAction::FlushJournal => "FlushJournal",
            TlaAction::SealManifest => "SealManifest",
            TlaAction::BeginCheckpoint { .. } => "BeginCheckpoint",
            TlaAction::SealCheckpoint => "SealCheckpoint",
            TlaAction::RetireHistoryPrefix { .. } => "RetireHistoryPrefix",
            TlaAction::DiscardDerivedState => "DiscardDerivedState",
            TlaAction::GracefulReopen { .. } => "GracefulReopen",
            TlaAction::BeginForkCreate => "BeginForkCreate",
            TlaAction::ForkNeedsReopen => "ForkNeedsReopen",
            TlaAction::ForkReopenBusy => "ForkReopenBusy",
            TlaAction::ForkBusyRetryClear => "ForkBusyRetryClear",
            TlaAction::ReopenForkToPublished => "ReopenForkToPublished",
            TlaAction::PublishForkDirect => "PublishForkDirect",
            TlaAction::CrashKill => "CrashKill",
            TlaAction::BeginRecover { .. } => "BeginRecover",
            TlaAction::AdvanceRecoverStage { .. } => "AdvanceRecoverStage",
            TlaAction::FinishRecover => "FinishRecover",
        },
        "postDurableHead": s.durable_head,
        "postCheckpointEpoch": s.checkpoint_epoch,
        "postStagedCheckpoint": s.staged_checkpoint,
        "postRetainedFloor": s.retained_floor,
        "postPendingWrite": pending_str(s.pending_write),
        "postActiveLease": lease_str(s.active_lease),
        "postForkPhase": fork_str(s.fork_phase),
        "postCrashed": s.crashed,
        "postRecoveryStage": recovery_str(s.recovery_stage),
        "postBuildId": build_str(s.build_id),
        "postDerivedState": derived_str(s.derived_state),
        "postRecoveredHead": s.recovered_head,
        "postReopenSucceeded": s.reopen_succeeded,
    });
    let map = obj.as_object_mut().unwrap();
    match action {
        TlaAction::BeginCheckpoint { epoch } => {
            map.insert("epoch".into(), json!(epoch));
        }
        TlaAction::RetireHistoryPrefix { new_floor } => {
            map.insert("newFloor".into(), json!(new_floor));
        }
        TlaAction::GracefulReopen { build_id } | TlaAction::BeginRecover { build_id } => {
            map.insert("buildId".into(), json!(build_str(build_id)));
        }
        TlaAction::AdvanceRecoverStage { resolved_head } => {
            map.insert("resolvedHead".into(), json!(resolved_head));
        }
        _ => {}
    }
    obj
}
