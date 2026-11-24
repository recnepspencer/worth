//! Live C1-C5 operations (clean commits, nested-lease retirement, staged/sealed
//! checkpoints, stand-alone retirement, D14 discard, and durable fork creation).
use super::super::super::super::graceful::signal::derived_discard::checkpoint_and_discard_world_home;
use super::driver::StoreDurabilityDst;
use super::faults::try_create_branch;
use super::observe::{
    assert_fork_head_ordinal, observe_branch_entity_payloads, observe_ref_entity_payloads,
    observe_root_branch, observe_root_retained_ordinals, ExpectedBranch,
};
use super::tla_oracle::{DerivedState, TlaAction};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store_application::ApplicationHomeCheckpointDenial;

impl StoreDurabilityDst {
    pub(super) fn op_clean_commit(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ctx = self.ctx("clean-commit");
        let round = self.next_round;
        self.next_round += 1;
        let do_nested_retire = self.oracle.state.retained_floor
            < self.oracle.state.checkpoint_epoch
            && self.oracle.state.checkpoint_epoch == self.oracle.state.durable_head
            && self.rng.range(2) == 0;

        let fx = self.fixture.as_ref().expect("live fixture");
        let root = observe_root_branch(fx, &ctx);
        let cancel = RuntimeWorldCancellationSource::new();

        let mut succ_commit: Option<CompositeCommitIdentity> = None;
        let mut nested_reclaimed: Option<(Vec<CompositeCommitIdentity>, Vec<u64>)> = None;
        exec.in_scope(|_| {
            let prepared =
                fx.prepare_publication(exec, root, crate::batches::create(round), &cancel.token());
            self.oracle.apply_and_verify(TlaAction::BeginCommit, &ctx);
            if do_nested_retire {
                let reclaimed = fx
                    .home
                    .world()
                    .recovery_port()
                    .retire_changed_history()
                    .unwrap_or_else(|e| {
                        panic!("{ctx}: nested-lease retire_changed_history failed: {e:?}")
                    });
                let retained_now = observe_root_retained_ordinals(fx, &ctx);
                nested_reclaimed = Some((reclaimed, retained_now));
            }
            let performed = fx.publish(exec, prepared, &cancel.token());
            let mut consumed = performed.consume();
            let succ = consumed.take_successor_observation().unwrap();
            succ_commit = Some(succ.selected_commit().clone());
        })
        .unwrap_or_else(|e| panic!("{ctx}: leased commit scope failed: {e:?}"));

        if let Some((reclaimed, retained_now)) = nested_reclaimed {
            self.observe_and_record_retirement(
                reclaimed,
                retained_now,
                &format!("{ctx}/nested-retire"),
            );
        }
        self.oracle.apply_and_verify(TlaAction::FlushJournal, &ctx);
        self.oracle.apply_and_verify(TlaAction::SealManifest, &ctx);

        let new_commit = succ_commit.expect("clean commit produced successor");
        let new_ord = new_commit.ordinal();
        assert!(
            new_ord > self.last_root_ordinal,
            "{ctx}: root commit ordinal must advance ({} -> {new_ord})",
            self.last_root_ordinal
        );
        self.last_root_ordinal = new_ord;
        self.epoch_ordinals
            .insert(self.oracle.state.durable_head, new_ord);

        self.mirror_ref_commit(round, exec);
        let expected_payloads = observe_ref_entity_payloads(&self.ref_runtime, exec);
        self.live_branches.insert(
            "Original World Root".to_owned(),
            ExpectedBranch {
                ordinal: new_ord,
                payloads: expected_payloads,
            },
        );
        self.verify_live_parity(exec, &ctx);
    }

    pub(super) fn op_checkpoint_sealed_or_crashed(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ep = self.oracle.state.durable_head;
        let inject_crash = self.rng.range(4) == 0;
        let ctx = self.ctx(if inject_crash {
            "checkpoint-staged-crash"
        } else {
            "checkpoint-seal"
        });
        self.oracle
            .apply_and_verify(TlaAction::BeginCheckpoint { epoch: ep }, &ctx);

        if inject_crash {
            let fail_sync = self
                .faults
                .as_ref()
                .expect("live faults")
                .fail_before_sync
                .clone();
            fail_sync.arm().unwrap();
            let cp_res = self.fixture.as_ref().unwrap().home.checkpoint(exec);
            assert!(
                fail_sync.is_consumed(),
                "{ctx}: FailBefore(SynchronizeFileState) did not fire during checkpoint"
            );
            assert!(
                cp_res.is_err(),
                "{ctx}: checkpoint unexpectedly succeeded despite consumed FailBefore fault"
            );
            self.stats.fault_checkpoint_fired += 1;
            self.faults = None;
            self.fixture = None;
            self.oracle.apply_and_verify(TlaAction::CrashKill, &ctx);
            self.recover_with_chained_crashes(None, exec, &ctx);
            return;
        }

        let fx = self.fixture.as_ref().expect("live fixture");
        let cp = fx
            .home
            .checkpoint(exec)
            .unwrap_or_else(|e| panic!("{ctx}: real checkpoint failed: {e:?}"));
        assert!(cp.generation() > 0, "{ctx}: checkpoint generation > 0");
        self.oracle
            .apply_and_verify(TlaAction::SealCheckpoint, &ctx);
        self.verify_live_parity(exec, &ctx);
    }

    pub(super) fn op_standalone_retire(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ctx = self.ctx("standalone-retire");
        let fx = self.fixture.as_ref().expect("live fixture");
        let reclaimed = fx
            .home
            .world()
            .recovery_port()
            .retire_changed_history()
            .unwrap_or_else(|e| panic!("{ctx}: retire_changed_history failed: {e:?}"));
        let retained_now = observe_root_retained_ordinals(fx, &ctx);
        self.observe_and_record_retirement(reclaimed, retained_now, &ctx);
        self.verify_live_parity(exec, &ctx);
    }

    pub(super) fn op_d14_discard(&mut self, exec: &ExecutionRequest<'_, '_>) {
        let ctx = self.ctx("d14-discard");
        let fx = self.fixture.as_ref().expect("live fixture");
        let (cp_gen, discard_gen) = checkpoint_and_discard_world_home(fx, exec);
        assert!(
            cp_gen > 0 && discard_gen == cp_gen + 1,
            "{ctx}: discard_gen ({discard_gen}) must advance cp_gen ({cp_gen}) + 1"
        );
        if self.oracle.state.checkpoint_epoch < self.oracle.state.durable_head {
            let ep = self.oracle.state.durable_head;
            self.oracle
                .apply_and_verify(TlaAction::BeginCheckpoint { epoch: ep }, &ctx);
            self.oracle
                .apply_and_verify(TlaAction::SealCheckpoint, &ctx);
        }
        self.oracle
            .apply_and_verify(TlaAction::DiscardDerivedState, &ctx);
        assert_eq!(
            self.oracle.state.derived_state,
            DerivedState::DiscardedBlocked
        );
        let blocked_cp = fx.home.checkpoint(exec);
        assert!(
            matches!(
                blocked_cp,
                Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded)
            ),
            "{ctx}: in-session checkpoint after D14 discard must return DerivedStateDiscarded"
        );
        self.stats.d14_blocks_verified += 1;

        if self.rng.range(2) == 0 {
            self.op_graceful_reopen(exec);
        } else {
            self.op_quiescent_crash_and_recover(exec);
        }
        assert_eq!(
            self.oracle.state.derived_state,
            DerivedState::DiscardedReopened
        );
        let fx2 = self.fixture.as_ref().expect("reopened fixture");
        assert_eq!(
            fx2.home.relational().cold_storage_reads().member_reads,
            0,
            "{ctx}: D14 reopen must never read discarded checkpoint members"
        );
        let insp = fx2
            .home
            .inspect_checkpoint(Default::default(), exec)
            .unwrap();
        assert!(
            insp.journals.iter().all(|j| j.covered == 0),
            "{ctx}: D14 reopen must report 0 checkpoint coverage"
        );
    }

    pub(super) fn op_durable_fork(&mut self, exec: &ExecutionRequest<'_, '_>) {
        if self.live_branches.len() >= 4 {
            let non_root: Vec<String> = self
                .live_branches
                .keys()
                .filter(|n| n.as_str() != "Original World Root")
                .cloned()
                .collect();
            let victim = self.rng.choose(&non_root).clone();
            let retire_ctx = self.ctx("retire-branch-for-cap");
            self.retire_live_branch(&victim, exec, &retire_ctx);
        }

        self.next_fork_seq += 1;
        let name = format!("s{}-f{}", self.seed, self.next_fork_seq);
        let prefix = format!("s{}-fp{}", self.seed, self.next_fork_seq);
        let native_fork = self.rng.range(2) == 0;
        let signal_fork = self.rng.range(2) == 0;
        let inject_fault = self.rng.range(2) == 0;
        let ctx = self.ctx(if inject_fault {
            "fork-interrupted-reopen"
        } else {
            "fork-direct"
        });
        let source_expected = self
            .live_branches
            .get("Original World Root")
            .expect("root branch tracked")
            .clone();

        self.oracle
            .apply_and_verify(TlaAction::BeginForkCreate, &ctx);

        if inject_fault {
            let fail_sync = self.faults.as_ref().unwrap().fail_before_sync.clone();
            fail_sync.arm().unwrap();
            let res = try_create_branch(
                self.fixture.as_ref().unwrap(),
                exec,
                &name,
                &prefix,
                native_fork,
                signal_fork,
                &ctx,
            );
            assert!(
                fail_sync.is_consumed(),
                "{ctx}: FailBefore(SynchronizeFileState) did not fire during fork creation"
            );
            assert!(
                res.is_err(),
                "{ctx}: fork creation unexpectedly succeeded despite consumed FailBefore fault"
            );
            self.stats.fault_fork_fired += 1;
            self.pending_fork = Some((name, prefix, native_fork, signal_fork, source_expected));
            self.faults = None;
            self.fixture = None;
            self.oracle.apply_and_verify(TlaAction::CrashKill, &ctx);
            self.recover_with_chained_crashes(None, exec, &ctx);

            let (fname, fprefix, nf, sf, expected_branch) = self.pending_fork.take().unwrap();
            let fx = self.fixture.as_ref().unwrap();
            let installed = fx
                .home
                .world()
                .observation_port()
                .observe_installed_product_branches(exec)
                .unwrap();
            let existing_ord = installed
                .branches()
                .iter()
                .find(|b| b.branch_identity().name().as_str() == fname)
                .map(|b| {
                    assert_eq!(
                        observe_branch_entity_payloads(fx, b, exec, &ctx),
                        expected_branch.payloads,
                        "{ctx}: recovered fork {fname:?} payload mismatch"
                    );
                    b.selected_commit().ordinal()
                });
            drop(installed);
            let mut saw_busy = false;
            let fork_ord = if let Some(ord) = existing_ord {
                assert_fork_head_ordinal(ord, expected_branch.ordinal, nf, sf, &ctx);
                ord
            } else {
                let (obs, busy) = try_create_branch(fx, exec, &fname, &fprefix, nf, sf, &ctx)
                    .unwrap_or_else(|e| {
                        panic!("{ctx}: fork retry after ReopenRequired blocked/failed: {e}")
                    });
                saw_busy = busy;
                let ord = obs.selected_commit().ordinal();
                assert_fork_head_ordinal(ord, expected_branch.ordinal, nf, sf, &ctx);
                assert_eq!(
                    observe_branch_entity_payloads(fx, &obs, exec, &ctx),
                    expected_branch.payloads
                );
                ord
            };
            if saw_busy {
                self.oracle
                    .apply_and_verify(TlaAction::ForkReopenBusy, &ctx);
                self.oracle
                    .apply_and_verify(TlaAction::ForkBusyRetryClear, &ctx);
            }
            self.live_branches.insert(
                fname,
                ExpectedBranch {
                    ordinal: fork_ord,
                    payloads: expected_branch.payloads,
                },
            );
            self.oracle
                .apply_and_verify(TlaAction::ReopenForkToPublished, &ctx);
            self.verify_live_parity(exec, &ctx);
            return;
        }

        let fx = self.fixture.as_ref().unwrap();
        let (obs, _saw_busy) =
            try_create_branch(fx, exec, &name, &prefix, native_fork, signal_fork, &ctx)
                .unwrap_or_else(|e| panic!("{ctx}: direct fork creation failed: {e}"));
        let fork_ord = obs.selected_commit().ordinal();
        assert_fork_head_ordinal(
            fork_ord,
            source_expected.ordinal,
            native_fork,
            signal_fork,
            &ctx,
        );
        assert_eq!(
            observe_branch_entity_payloads(fx, &obs, exec, &ctx),
            source_expected.payloads
        );
        self.live_branches.insert(
            name.clone(),
            ExpectedBranch {
                ordinal: fork_ord,
                payloads: source_expected.payloads,
            },
        );
        self.oracle
            .apply_and_verify(TlaAction::PublishForkDirect, &ctx);
        drop(obs);

        if self.rng.range(3) == 0 {
            self.retire_live_branch(&name, exec, &format!("{ctx}/retire"));
        }
        self.verify_live_parity(exec, &ctx);
    }

    fn retire_live_branch(&mut self, name: &str, exec: &ExecutionRequest<'_, '_>, ctx: &str) {
        let fx = self.fixture.as_ref().unwrap();
        let installed = fx
            .home
            .world()
            .observation_port()
            .observe_installed_product_branches(exec)
            .unwrap();
        let target = installed
            .branches()
            .iter()
            .find(|b| b.branch_identity().name().as_str() == name)
            .unwrap_or_else(|| panic!("{ctx}: missing live branch {name:?}"))
            .clone();
        drop(installed);
        let report = fx
            .home
            .world()
            .branch_port()
            .retire_product_branch(&target)
            .unwrap_or_else(|e| panic!("{ctx}: retire_product_branch failed: {e:?}"));
        drop(target);
        let release = fx
            .home
            .world()
            .recovery_port()
            .release_retired_branch_history(report.retired_head())
            .unwrap_or_else(|e| panic!("{ctx}: release_retired_branch_history failed: {e:?}"));
        assert!(
            release.held_commit().is_none(),
            "{ctx}: retired branch history release left unexpected held commit {:?}",
            release.held_commit()
        );
        for c in release.reclaimed_commits() {
            self.retired_ordinals.insert(c.ordinal());
        }
        self.live_branches.remove(name);
    }
}
