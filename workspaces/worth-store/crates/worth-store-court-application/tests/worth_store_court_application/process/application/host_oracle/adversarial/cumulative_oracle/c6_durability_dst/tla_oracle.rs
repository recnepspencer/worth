//! Direct Rust translation of `specs/tla/store_m3/StoreDurability.tla`.
use super::tla_trace::encode_trace_row;
pub(super) use super::tla_trace::{
    BuildTag, DerivedState, ForkPhase, LeaseMode, PendingWrite, RecoveryStage, TlaAction,
    TlaModelState,
};
use serde_json::Value;
use std::path::Path;

pub(super) const MAX_COMMITS: u64 = 64;

pub(super) struct TlaOracle {
    pub state: TlaModelState,
    record_trace: bool,
    trace_rows: Vec<Value>,
    step_count: usize,
}

impl TlaOracle {
    pub(super) fn new(record_trace: bool) -> Self {
        let oracle = Self {
            state: TlaModelState::default(),
            record_trace,
            trace_rows: Vec::new(),
            step_count: 0,
        };
        oracle.assert_invariants("Init");
        oracle
    }

    pub(super) fn step_count(&self) -> usize {
        self.step_count
    }

    pub(super) fn apply_and_verify(&mut self, action: TlaAction, ctx: &str) {
        let prev = self.state;
        apply_step(&mut self.state, action, ctx);
        self.assert_invariants(ctx);
        self.assert_monotonic(&prev, ctx);
        self.step_count += 1;
        if self.record_trace {
            self.trace_rows.push(encode_trace_row(action, self.state));
        }
    }

    fn assert_invariants(&self, ctx: &str) {
        let s = self.state;
        assert!(s.durable_head <= MAX_COMMITS, "{ctx}: TypeOK durableHead");
        let staged_ok = s.staged_checkpoint == 0
            || (s.staged_checkpoint <= s.durable_head
                && (s.staged_checkpoint > s.checkpoint_epoch
                    || (s.derived_state == DerivedState::DiscardedReopened
                        && s.staged_checkpoint == s.checkpoint_epoch
                        && s.staged_checkpoint > 0)));
        assert!(
            s.retained_floor <= s.checkpoint_epoch
                && s.checkpoint_epoch <= s.durable_head
                && staged_ok
                && s.reopen_succeeded,
            "{ctx}: NoRetiredHistoryGap violated: {s:?}"
        );
        assert!(
            s.recovered_head <= s.durable_head,
            "{ctx}: DurabilityNoLoss violated: {s:?}"
        );
        assert!(
            (s.pending_write == PendingWrite::None) == (s.pending_target == 0)
                && (s.pending_write == PendingWrite::None) == (s.active_lease == LeaseMode::None)
                && (!s.crashed || s.staged_checkpoint == 0)
                && (s.crashed || s.recovery_stage == RecoveryStage::None)
                && (!(s.crashed && s.recovery_stage == RecoveryStage::ManifestSwitch)
                    || (s.pending_write == PendingWrite::None
                        && s.active_lease == LeaseMode::None)),
            "{ctx}: AtomicityCleanState violated: {s:?}"
        );
        assert!(
            s.derived_state != DerivedState::DiscardedBlocked || s.staged_checkpoint == 0,
            "{ctx}: D14BlocksInSessionCheckpoint violated: {s:?}"
        );
    }

    fn assert_monotonic(&self, prev: &TlaModelState, ctx: &str) {
        let s = self.state;
        assert!(
            s.durable_head >= prev.durable_head
                && s.checkpoint_epoch >= prev.checkpoint_epoch
                && s.retained_floor >= prev.retained_floor
                && s.recovered_head >= prev.recovered_head,
            "{ctx}: MonotonicDurability violated: prev={prev:?} -> next={s:?}"
        );
    }

    pub(super) fn write_ndjson(&self, path: &Path) {
        let mut out = String::new();
        for row in &self.trace_rows {
            out.push_str(&serde_json::to_string(row).expect("serialize trace row"));
            out.push('\n');
        }
        std::fs::write(path, out).unwrap_or_else(|e| panic!("write trace {}: {e}", path.display()));
    }
}

fn apply_step(s: &mut TlaModelState, action: TlaAction, ctx: &str) {
    match action {
        TlaAction::BeginCommit => {
            assert!(
                !s.crashed
                    && s.pending_write == PendingWrite::None
                    && s.staged_checkpoint == 0
                    && s.active_lease == LeaseMode::None
                    && s.durable_head < MAX_COMMITS,
                "{ctx}: BeginCommit precondition violated: {s:?}"
            );
            s.pending_write = PendingWrite::JournalUnflushed;
            s.pending_target = s.durable_head + 1;
            s.active_lease = LeaseMode::CommitLease;
        }
        TlaAction::FlushJournal => {
            assert!(
                !s.crashed && s.pending_write == PendingWrite::JournalUnflushed,
                "{ctx}: FlushJournal precondition violated: {s:?}"
            );
            s.pending_write = PendingWrite::JournalFlushedPreManifest;
        }
        TlaAction::SealManifest => {
            assert!(
                !s.crashed && s.pending_write == PendingWrite::JournalFlushedPreManifest,
                "{ctx}: SealManifest precondition violated: {s:?}"
            );
            s.durable_head = s.pending_target;
            s.pending_write = PendingWrite::None;
            s.pending_target = 0;
            s.active_lease = LeaseMode::None;
        }
        TlaAction::BeginCheckpoint { epoch } => {
            let min_ep =
                if s.derived_state == DerivedState::DiscardedReopened && s.checkpoint_epoch > 0 {
                    s.checkpoint_epoch
                } else {
                    s.checkpoint_epoch + 1
                };
            assert!(
                !s.crashed
                    && s.pending_write == PendingWrite::None
                    && s.staged_checkpoint == 0
                    && s.derived_state != DerivedState::DiscardedBlocked
                    && epoch >= min_ep
                    && epoch <= s.durable_head,
                "{ctx}: BeginCheckpoint({epoch}) precondition violated: {s:?}"
            );
            s.staged_checkpoint = epoch;
        }
        TlaAction::SealCheckpoint => {
            assert!(
                !s.crashed
                    && s.staged_checkpoint > 0
                    && s.staged_checkpoint <= s.durable_head
                    && (s.staged_checkpoint > s.checkpoint_epoch
                        || s.derived_state == DerivedState::DiscardedReopened),
                "{ctx}: SealCheckpoint precondition violated: {s:?}"
            );
            s.checkpoint_epoch = s.staged_checkpoint;
            s.staged_checkpoint = 0;
            s.derived_state = DerivedState::Covered;
        }
        TlaAction::RetireHistoryPrefix { new_floor } => {
            assert!(
                !s.crashed && new_floor > s.retained_floor && new_floor <= s.checkpoint_epoch,
                "{ctx}: RetireHistoryPrefix({new_floor}) violated sealed checkpointEpoch={}: {s:?}",
                s.checkpoint_epoch
            );
            s.retained_floor = new_floor;
        }
        TlaAction::DiscardDerivedState => {
            assert!(
                !s.crashed
                    && s.pending_write == PendingWrite::None
                    && s.staged_checkpoint == 0
                    && s.derived_state == DerivedState::Covered,
                "{ctx}: DiscardDerivedState precondition violated: {s:?}"
            );
            s.derived_state = DerivedState::DiscardedBlocked;
        }
        TlaAction::GracefulReopen { build_id } => {
            assert!(
                !s.crashed
                    && s.pending_write == PendingWrite::None
                    && s.staged_checkpoint == 0
                    && s.active_lease == LeaseMode::None,
                "{ctx}: GracefulReopen precondition violated: {s:?}"
            );
            s.build_id = build_id;
            if s.derived_state == DerivedState::DiscardedBlocked {
                s.derived_state = DerivedState::DiscardedReopened;
            }
            s.recovered_head = s.durable_head;
            s.reopen_succeeded =
                s.retained_floor <= s.checkpoint_epoch && s.checkpoint_epoch <= s.durable_head;
        }
        TlaAction::BeginForkCreate => {
            assert!(
                !s.crashed && matches!(s.fork_phase, ForkPhase::Idle | ForkPhase::Published),
                "{ctx}: BeginForkCreate precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::Creating;
        }
        TlaAction::ForkNeedsReopen => {
            assert!(
                !s.crashed && s.fork_phase == ForkPhase::Creating,
                "{ctx}: ForkNeedsReopen precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::ReopenRequired;
        }
        TlaAction::ForkReopenBusy => {
            assert!(
                !s.crashed && s.fork_phase == ForkPhase::ReopenRequired,
                "{ctx}: ForkReopenBusy precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::ReopenBusy;
        }
        TlaAction::ForkBusyRetryClear => {
            assert!(
                !s.crashed && s.fork_phase == ForkPhase::ReopenBusy,
                "{ctx}: ForkBusyRetryClear precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::ReopenRequired;
        }
        TlaAction::ReopenForkToPublished => {
            assert!(
                !s.crashed && s.fork_phase == ForkPhase::ReopenRequired,
                "{ctx}: ReopenForkToPublished precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::Published;
        }
        TlaAction::PublishForkDirect => {
            assert!(
                !s.crashed && s.fork_phase == ForkPhase::Creating,
                "{ctx}: PublishForkDirect precondition violated: {s:?}"
            );
            s.fork_phase = ForkPhase::Published;
        }
        TlaAction::CrashKill => {
            assert!(
                !s.crashed
                    || matches!(
                        s.recovery_stage,
                        RecoveryStage::Staging | RecoveryStage::ManifestSwitch
                    ),
                "{ctx}: CrashKill precondition violated: {s:?}"
            );
            s.crashed = true;
            s.recovery_stage = RecoveryStage::None;
            s.staged_checkpoint = 0;
            if matches!(s.fork_phase, ForkPhase::Creating | ForkPhase::ReopenBusy) {
                s.fork_phase = ForkPhase::ReopenRequired;
            }
        }
        TlaAction::BeginRecover { build_id } => {
            assert!(
                s.crashed && s.recovery_stage == RecoveryStage::None,
                "{ctx}: BeginRecover precondition violated: {s:?}"
            );
            s.recovery_stage = RecoveryStage::Staging;
            s.build_id = build_id;
        }
        TlaAction::AdvanceRecoverStage { resolved_head } => {
            assert!(
                s.crashed && s.recovery_stage == RecoveryStage::Staging,
                "{ctx}: AdvanceRecoverStage precondition violated: {s:?}"
            );
            let allowed = match s.pending_write {
                PendingWrite::JournalFlushedPreManifest => {
                    resolved_head == s.durable_head || resolved_head == s.pending_target
                }
                PendingWrite::None | PendingWrite::JournalUnflushed => {
                    resolved_head == s.durable_head
                }
            };
            assert!(
                allowed,
                "{ctx}: AdvanceRecoverStage({resolved_head}) invalid for state {s:?}"
            );
            s.durable_head = resolved_head;
            s.pending_write = PendingWrite::None;
            s.pending_target = 0;
            s.active_lease = LeaseMode::None;
            s.recovery_stage = RecoveryStage::ManifestSwitch;
        }
        TlaAction::FinishRecover => {
            assert!(
                s.crashed && s.recovery_stage == RecoveryStage::ManifestSwitch,
                "{ctx}: FinishRecover precondition violated: {s:?}"
            );
            if s.derived_state == DerivedState::DiscardedBlocked {
                s.derived_state = DerivedState::DiscardedReopened;
            }
            s.recovered_head = s.durable_head;
            s.reopen_succeeded =
                s.retained_floor <= s.checkpoint_epoch && s.checkpoint_epoch <= s.durable_head;
            s.crashed = false;
            s.recovery_stage = RecoveryStage::None;
        }
    }
}
