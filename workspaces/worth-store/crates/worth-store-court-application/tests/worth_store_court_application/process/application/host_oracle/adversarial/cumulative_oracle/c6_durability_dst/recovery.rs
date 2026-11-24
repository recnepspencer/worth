//! Fault-injected commits, graceful close/reopen, and multi-step interruptible
//! + chained crash recovery for the C6 Store Durability DST.
use super::driver::StoreDurabilityDst;
use super::faults::{
    assert_physical_io_open_refusal, drive_open_outcome, drive_publish_outcome,
    open_world_home_with_faults, pending_custody_handles, settle_pending_custody,
    try_open_with_recovery_fault,
};
use super::observe::{
    assert_fault_prepare_refusal, assert_fault_publish_outcome, observe_ref_entity_payloads,
    observe_root_branch, observe_root_entity_payloads, ExpectedBranch,
};
use super::tla_oracle::{DerivedState, ForkPhase, PendingWrite, TlaAction};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_application::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};

impl StoreDurabilityDst {
    pub(super) fn op_fault_commit_and_recover(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let indeterminate = self.rng.range(2) == 0;
        let ctx = self.ctx(if indeterminate {
            "fault-commit-indeterminate"
        } else {
            "fault-commit-fail-before"
        });
        let round = self.next_round;
        self.next_round += 1;

        if indeterminate {
            self.inject_indeterminate_commit_fault(round, exec, &ctx);
        } else {
            self.inject_fail_before_write_commit_fault(round, exec, &ctx);
        }

        self.faults = None;
        self.fixture = None;
        self.oracle.apply_and_verify(TlaAction::CrashKill, &ctx);
        self.recover_with_chained_crashes(Some(round), exec, &ctx);
    }

    fn inject_fail_before_write_commit_fault(
        &mut self,
        round: usize,
        exec: &ExecutionRequest<'_, '_>,
        ctx: &str,
    ) {
        let fx = self.fixture.as_ref().expect("live fixture");
        let fail_write = self
            .faults
            .as_ref()
            .expect("live faults")
            .fail_before_write
            .clone();
        let root = observe_root_branch(fx, ctx);
        let cancel = RuntimeWorldCancellationSource::new();
        let port = fx.home.world().publication_port();
        let change = worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary();

        fail_write.arm().unwrap();
        self.oracle.apply_and_verify(TlaAction::BeginCommit, ctx);

        let open_out = port.open_publication_without_signal(
            exec,
            root.clone(),
            fx.admit_initial_program_source(&root),
            change.clone(),
            &cancel.token(),
            None,
        );
        match drive_open_outcome(open_out, exec, ctx) {
            WorldOpenAttemptOutcome::Refused(WorldOpenAttemptRefusal::Durable(_))
            | WorldOpenAttemptOutcome::Unresolved(_)
            | WorldOpenAttemptOutcome::EvidenceRequired(_) => {}
            WorldOpenAttemptOutcome::Opened(opened) => {
                let native = fx.home.relational().runtime();
                let mut tx = native
                    .begin_branch_transaction(root.basis().relational_basis(), change)
                    .unwrap_or_else(|e| panic!("{ctx}: begin_branch_transaction failed: {e:?}"));
                tx.push_batch(crate::batches::create(round), exec.allocation_policy())
                    .unwrap_or_else(|e| panic!("{ctx}: push_batch failed: {e:?}"));
                let prop = tx
                    .validate(exec, native)
                    .unwrap_or_else(|e| panic!("{ctx}: validate failed: {e:?}"));
                match opened.prepare_native(native, exec, prop) {
                    Err(refusal) => assert_fault_prepare_refusal(refusal.cause(), false, ctx),
                    Ok(prepared) => {
                        let out = drive_publish_outcome(
                            port.execute_durable_without_signal(*exec, prepared, &cancel.token()),
                            exec,
                            ctx,
                        );
                        assert_fault_publish_outcome(&out, ctx);
                    }
                }
            }
            other => panic!("{ctx}: unexpected open_publication outcome: {other:?}"),
        }
        assert!(
            fail_write.is_consumed(),
            "{ctx}: FailBefore(PositionedWrite) fault did not fire during commit"
        );
        self.stats.fault_write_fired += 1;
    }

    fn inject_indeterminate_commit_fault(
        &mut self,
        round: usize,
        exec: &ExecutionRequest<'_, '_>,
        ctx: &str,
    ) {
        let fx = self.fixture.as_ref().expect("live fixture");
        let indet_sync = self
            .faults
            .as_ref()
            .expect("live faults")
            .indeterminate_sync
            .clone();
        let root = observe_root_branch(fx, ctx);
        let cancel = RuntimeWorldCancellationSource::new();
        let port = fx.home.world().publication_port();
        let change = worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary();

        let opened = match drive_open_outcome(
            port.open_publication_without_signal(
                exec,
                root.clone(),
                fx.admit_initial_program_source(&root),
                change.clone(),
                &cancel.token(),
                None,
            ),
            exec,
            ctx,
        ) {
            WorldOpenAttemptOutcome::Opened(opened) => opened,
            other => panic!("{ctx}: clean open_publication before sync fault failed: {other:?}"),
        };
        let native = fx.home.relational().runtime();
        let mut tx = native
            .begin_branch_transaction(root.basis().relational_basis(), change)
            .unwrap_or_else(|e| panic!("{ctx}: begin_branch_transaction failed: {e:?}"));
        tx.push_batch(crate::batches::create(round), exec.allocation_policy())
            .unwrap_or_else(|e| panic!("{ctx}: push_batch failed: {e:?}"));
        let prop = tx
            .validate(exec, native)
            .unwrap_or_else(|e| panic!("{ctx}: validate failed: {e:?}"));

        indet_sync.arm().unwrap();
        self.oracle.apply_and_verify(TlaAction::BeginCommit, ctx);

        match opened.prepare_native(native, exec, prop) {
            Err(refusal) => assert_fault_prepare_refusal(refusal.cause(), true, ctx),
            Ok(prepared) => {
                let out = drive_publish_outcome(
                    port.execute_durable_without_signal(*exec, prepared, &cancel.token()),
                    exec,
                    ctx,
                );
                assert_fault_publish_outcome(&out, ctx);
            }
        }
        assert!(
            indet_sync.is_consumed(),
            "{ctx}: IndeterminateAfterEffect(SynchronizeFileState) did not fire during commit"
        );
        self.stats.fault_indeterminate_fired += 1;
        self.oracle.apply_and_verify(TlaAction::FlushJournal, ctx);
    }

    pub(super) fn op_graceful_reopen(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ctx = self.ctx("graceful-reopen");
        let fx = self.fixture.take().expect("live fixture");
        self.faults = None;
        let blocked = self.oracle.state.derived_state == DerivedState::DiscardedBlocked;
        let closed = fx
            .home
            .close(exec)
            .unwrap_or_else(|e| panic!("{ctx}: close refused: {:?}", e.reason()));
        match closed {
            ApplicationHomeCloseOutcome::Closed {
                checkpoint: Ok(_), ..
            } => {
                assert!(!blocked, "{ctx}: close must not seal checkpoint under D14");
                if self.oracle.state.checkpoint_epoch < self.oracle.state.durable_head
                    || (self.oracle.state.derived_state == DerivedState::DiscardedReopened
                        && self.oracle.state.checkpoint_epoch > 0)
                {
                    let ep = self.oracle.state.durable_head;
                    self.oracle
                        .apply_and_verify(TlaAction::BeginCheckpoint { epoch: ep }, &ctx);
                    self.oracle
                        .apply_and_verify(TlaAction::SealCheckpoint, &ctx);
                }
            }
            ApplicationHomeCloseOutcome::Closed {
                checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                ..
            } => {
                assert!(blocked, "{ctx}: unexpected DerivedStateDiscarded on close");
            }
            other => panic!(
                "{ctx}: unexpected close outcome: {}",
                std::any::type_name_of_val(&other)
            ),
        }

        let next_build = self.next_other_build();
        let sync_nth = 1 + (self.rng.range(2) as u64);
        let (reopened, faults) =
            open_world_home_with_faults(&self.home_dir, next_build, sync_nth, exec);
        self.fixture = Some(reopened);
        self.faults = Some(faults);
        self.oracle.apply_and_verify(
            TlaAction::GracefulReopen {
                build_id: next_build,
            },
            &ctx,
        );
        self.verify_live_parity(exec, &ctx);
    }

    pub(super) fn op_quiescent_crash_and_recover(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ctx = self.ctx("quiescent-crash");
        self.faults = None;
        self.fixture = None;
        self.oracle.apply_and_verify(TlaAction::CrashKill, &ctx);
        self.recover_with_chained_crashes(None, exec, &ctx);
    }

    pub(super) fn recover_with_chained_crashes(
        &mut self,
        inflight_round: Option<usize>,
        exec: &ExecutionRequest<'_, '_>,
        ctx: &str,
    ) {
        if self.rng.range(2) == 0 {
            let rec_build = self.next_other_build();
            let role = if self.rng.range(2) == 0 {
                MediaOperationRole::PositionedRead
            } else {
                MediaOperationRole::OpenExisting
            };
            let (res, consumed) =
                try_open_with_recovery_fault(&self.home_dir, rec_build, role, 1, exec);
            assert!(
                consumed,
                "{ctx}: recovery staging fault {role:?}#1 did not fire"
            );
            let err = res.err().unwrap_or_else(|| {
                panic!("{ctx}: reopen succeeded despite consumed {role:?} fault")
            });
            assert_physical_io_open_refusal(&err, ctx);
            self.stats.fault_recovery_staging_fired += 1;
            self.oracle.apply_and_verify(
                TlaAction::BeginRecover {
                    build_id: rec_build,
                },
                &format!("{ctx}/chained-stage1"),
            );
            self.oracle
                .apply_and_verify(TlaAction::CrashKill, &format!("{ctx}/chained-kill1"));
        }

        let rec_build = self.next_other_build();
        self.oracle.apply_and_verify(
            TlaAction::BeginRecover {
                build_id: rec_build,
            },
            &format!("{ctx}/recover-begin"),
        );
        let sync_nth = 1 + (self.rng.range(2) as u64);
        let (reopened, faults) =
            open_world_home_with_faults(&self.home_dir, rec_build, sync_nth, exec);
        let had_pending_custody = !pending_custody_handles(&reopened, ctx).is_empty();
        let settled = settle_pending_custody(&reopened, exec, ctx);

        let observed_ord = observe_root_branch(&reopened, ctx)
            .selected_commit()
            .ordinal();
        let observed_payloads = observe_root_entity_payloads(&reopened, exec, ctx);
        let resolved_head = if observed_ord > self.last_root_ordinal {
            assert_eq!(
                self.oracle.state.pending_write,
                PendingWrite::JournalFlushedPreManifest,
                "{ctx}: root ordinal advanced without flushed pending write"
            );
            let round =
                inflight_round.expect("ordinal advanced must correspond to in-flight commit round");
            self.mirror_ref_commit(round, exec);
            let expected_payloads = observe_ref_entity_payloads(&self.ref_runtime, exec);
            assert_eq!(
                observed_payloads, expected_payloads,
                "{ctx}: recovered committed entity payload mismatch vs MemoryOnly"
            );
            self.last_root_ordinal = observed_ord;
            let new_head = self.oracle.state.durable_head + 1;
            self.epoch_ordinals.insert(new_head, observed_ord);
            self.live_branches.insert(
                "Original World Root".to_owned(),
                ExpectedBranch {
                    ordinal: observed_ord,
                    payloads: expected_payloads,
                },
            );
            new_head
        } else {
            assert_eq!(
                observed_ord, self.last_root_ordinal,
                "{ctx}: root commit ordinal regressed ({observed_ord} < {})",
                self.last_root_ordinal
            );
            assert_eq!(
                observed_payloads,
                observe_ref_entity_payloads(&self.ref_runtime, exec),
                "{ctx}: recovered pre-op entity payload mismatch vs MemoryOnly"
            );
            self.oracle.state.durable_head
        };

        self.oracle.apply_and_verify(
            TlaAction::AdvanceRecoverStage { resolved_head },
            &format!("{ctx}/recover-advance"),
        );
        self.oracle
            .apply_and_verify(TlaAction::FinishRecover, &format!("{ctx}/recover-finish"));
        if settled.saw_fork_busy && self.oracle.state.fork_phase == ForkPhase::ReopenRequired {
            self.oracle
                .apply_and_verify(TlaAction::ForkReopenBusy, &format!("{ctx}/fork-busy"));
            self.oracle
                .apply_and_verify(TlaAction::ForkBusyRetryClear, &format!("{ctx}/fork-clear"));
        }

        // If the pending fork was already completed by custody settlement, install it
        // in expected live_branches only when observed on reopen (handled in op_durable_fork).
        if self.pending_fork.is_none() {
            self.fixture = Some(reopened);
            self.faults = Some(faults);
            self.verify_live_parity(exec, ctx);
            if !had_pending_custody && self.rng.range(3) == 0 {
                self.faults = None;
                self.fixture = None;
                self.oracle
                    .apply_and_verify(TlaAction::CrashKill, &format!("{ctx}/followup-crash"));
                let b3 = self.next_other_build();
                self.oracle.apply_and_verify(
                    TlaAction::BeginRecover { build_id: b3 },
                    &format!("{ctx}/followup-begin"),
                );
                let sync_nth2 = 1 + (self.rng.range(2) as u64);
                let (reopened2, faults2) =
                    open_world_home_with_faults(&self.home_dir, b3, sync_nth2, exec);
                let _ = settle_pending_custody(&reopened2, exec, ctx);
                assert_eq!(
                    observe_root_branch(&reopened2, ctx)
                        .selected_commit()
                        .ordinal(),
                    self.last_root_ordinal,
                    "{ctx}: follow-up crash flipped or regressed root commit ordinal"
                );
                self.oracle.apply_and_verify(
                    TlaAction::AdvanceRecoverStage { resolved_head },
                    &format!("{ctx}/followup-advance"),
                );
                self.oracle
                    .apply_and_verify(TlaAction::FinishRecover, &format!("{ctx}/followup-finish"));
                self.fixture = Some(reopened2);
                self.faults = Some(faults2);
                self.verify_live_parity(exec, &format!("{ctx}/followup"));
            }
        } else {
            self.fixture = Some(reopened);
            self.faults = Some(faults);
        }
    }
}
