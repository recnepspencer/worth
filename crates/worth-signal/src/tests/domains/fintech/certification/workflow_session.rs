use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::facade::{
    AspectVersion, LineageRecord, ReplayEventKind, ReplaySlice, SignalBranchHandle, SignalError,
    SignalRuntimePolicy, SignalSnapshotV1,
};

use super::super::aspects::{ALERT, CURVE, LIQUIDITY, PRICE, RISK, VOL};
use super::super::audit_surface::PrimaryAuditSurface;
use super::super::fixture::FintechWorld;
use super::super::regimes::MarketRegime;
use worth_harness::facade::WorkflowRuntimeProfile;

#[derive(Debug, Clone)]
pub(super) enum FintechWorkflowStep {
    SeedRegime {
        regime: MarketRegime,
        seed: u64,
    },
    CaptureActiveSnapshot {
        alias: &'static str,
    },
    OpenBranch {
        branch_name: &'static str,
        alias: &'static str,
    },
    SwitchBranch {
        alias: &'static str,
    },
    ReadPrimaryAuditSurface {
        alias: &'static str,
    },
    InjectSyntheticRollback,
    RestoreSnapshot {
        branch_alias: &'static str,
        snapshot_alias: &'static str,
    },
    CaptureReplay {
        branch_alias: &'static str,
        alias: &'static str,
    },
    CaptureReplayAroundSnapshot {
        snapshot_alias: &'static str,
        alias: &'static str,
    },
    CaptureMainRiskLineage {
        alias: &'static str,
    },
}

pub(super) struct CertifiedFintechWorkflowSession {
    pub(super) world: FintechWorld,
    pub(super) workers: Option<usize>,
    pub(super) policy: SignalRuntimePolicy,
    pub(super) named_branches: BTreeMap<String, SignalBranchHandle>,
    pub(super) named_snapshots: BTreeMap<String, SignalSnapshotV1>,
    pub(super) named_audits: BTreeMap<String, PrimaryAuditSurface>,
    pub(super) named_replays: BTreeMap<String, ReplaySlice>,
    pub(super) named_lineages: BTreeMap<String, Vec<LineageRecord>>,
    pub(super) executed_steps: Vec<String>,
    pub(super) checkpoints: Vec<String>,
    pub(super) failure_injections: Vec<String>,
}

impl CertifiedFintechWorkflowSession {
    pub(super) fn branch(&self, alias: &str) -> Result<SignalBranchHandle, SignalError> {
        self.named_branches.get(alias).cloned().ok_or_else(|| {
            SignalError::invalid_input(format!("unknown certified fintech branch alias `{alias}`"))
        })
    }

    pub(super) fn snapshot(&self, alias: &str) -> Result<SignalSnapshotV1, SignalError> {
        self.named_snapshots.get(alias).cloned().ok_or_else(|| {
            SignalError::invalid_input(format!(
                "unknown certified fintech snapshot alias `{alias}`"
            ))
        })
    }

    pub(super) fn replay(&self, alias: &str) -> Result<&ReplaySlice, SignalError> {
        self.named_replays.get(alias).ok_or_else(|| {
            SignalError::invalid_input(format!("unknown certified fintech replay alias `{alias}`"))
        })
    }

    pub(super) fn lineage(&self, alias: &str) -> Result<&Vec<LineageRecord>, SignalError> {
        self.named_lineages.get(alias).ok_or_else(|| {
            SignalError::invalid_input(format!("unknown certified fintech lineage alias `{alias}`"))
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct SignalFintechWorkflowCertificationAdapter;

impl SignalFintechWorkflowCertificationAdapter {
    pub(super) fn runtime_policy(
        profile: &WorkflowRuntimeProfile,
    ) -> Result<SignalRuntimePolicy, SignalError> {
        match profile.policy_name.as_deref().unwrap_or("fintech") {
            "fintech" => Ok(SignalRuntimePolicy::fintech()
                .with_history_limit(8)
                .with_detail_limit(4)),
            "development" => Ok(SignalRuntimePolicy::development()
                .with_history_limit(8)
                .with_detail_limit(4)),
            "forensic" => Ok(SignalRuntimePolicy::forensic()
                .with_history_limit(8)
                .with_detail_limit(4)),
            other => Err(SignalError::invalid_input(format!(
                "unsupported workflow certification policy `{other}`"
            ))),
        }
    }

    pub(super) fn workers(profile: &WorkflowRuntimeProfile) -> Result<Option<usize>, SignalError> {
        match profile.executor_name.as_deref().unwrap_or("serial") {
            "serial" => Ok(None),
            "lease-1" => Ok(Some(1)),
            "lease-2" => Ok(Some(2)),
            "lease-4" => Ok(Some(4)),
            other => Err(SignalError::invalid_input(format!(
                "unsupported workflow certification executor `{other}`"
            ))),
        }
    }

    pub(super) fn version_summary(version: &AspectVersion) -> Value {
        json!({
            "price": version.get(PRICE),
            "vol": version.get(VOL),
            "curve": version.get(CURVE),
            "liquidity": version.get(LIQUIDITY),
            "risk": version.get(RISK),
            "alert": version.get(ALERT),
        })
    }

    pub(super) fn replay_summary(replay: &ReplaySlice) -> Value {
        let mut kinds = BTreeMap::new();
        for frame in &replay.frames {
            *kinds.entry(format!("{:?}", frame.kind)).or_insert(0usize) += 1;
        }
        json!({
            "frame_count": replay.frames.len(),
            "start": replay.start.map(|cursor| cursor.0),
            "end": replay.end.map(|cursor| cursor.0),
            "kinds": kinds,
        })
    }

    pub(super) fn lineage_summary(lineage: &[LineageRecord]) -> Value {
        let mut events = BTreeMap::new();
        for record in lineage {
            *events.entry(record.label().to_string()).or_insert(0usize) += 1;
        }
        json!({
            "record_count": lineage.len(),
            "events": events,
        })
    }

    pub(super) fn parse_replay_kind(value: &str) -> Result<ReplayEventKind, SignalError> {
        match value {
            "TaskApplied" => Ok(ReplayEventKind::TaskApplied),
            "TransactionCommitted" => Ok(ReplayEventKind::TransactionCommitted),
            "TransactionRolledBack" => Ok(ReplayEventKind::TransactionRolledBack),
            "FailureRecorded" => Ok(ReplayEventKind::FailureRecorded),
            "SnapshotCaptured" => Ok(ReplayEventKind::SnapshotCaptured),
            "SnapshotRestored" => Ok(ReplayEventKind::SnapshotRestored),
            "BranchCreated" => Ok(ReplayEventKind::BranchCreated),
            "BranchSwitched" => Ok(ReplayEventKind::BranchSwitched),
            other => Err(SignalError::invalid_input(format!(
                "unknown replay event kind `{other}`"
            ))),
        }
    }

    pub(super) fn parse_lineage_events(value: &str) -> Result<Vec<String>, SignalError> {
        value
            .split(',')
            .map(|event| match event {
                "Refreshed"
                | "Replaced"
                | "Restored"
                | "BranchedFrom"
                | "BranchSwitched"
                | "MergedFrom"
                | "MemoizedReuse"
                | "InvalidatedWithoutReplacement" => Ok(event.to_string()),
                other => Err(SignalError::invalid_input(format!(
                    "unknown lineage event `{other}`"
                ))),
            })
            .collect()
    }
}
