//! Core Query Host cumulative step helpers (commits, receipt-bound retry+drift,
//! mid-op Store mutate/checkpoint/close seam kills + recovery-stage kills, and
//! post-cumulative custody release verified against the live MemoryOnly ref).
use super::super::super::super::{protocol::Request, session::Session};
use super::super::super::dimensions_matrix::verify::{close_normal, inspect_cadence};
use super::super::super::recovery_fixtures::MutateSeamOrdinals;
use super::super::super::recovery_interruption_oracle::{opening_stage_index, PHYSICAL_STAGES};
use super::super::super::seam_kill_sweep::{kill_at_seam_with_ctx, ALL_SEAMS};
use super::super::{SplitMix64, StepCtx};
use super::release::kill_recovery_with_build_ctx;
use super::HostOracleState;
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage as PhysStage;
use worth_store_application::{
    ApplicationHomeOpeningEdge as OpenEdge, ApplicationHomeOpeningStage as OpenStage,
};
use worth_store_testkit_application_oracle::facade::host_observation::{
    comparison,
    receipts::{self, MutationFate},
    requests,
};

fn recovery_cut_spec(rng: &mut SplitMix64, has_dirty_wal: bool) -> String {
    if has_dirty_wal && rng.range(2) == 0 {
        let stages: Vec<PhysStage> = PHYSICAL_STAGES
            .into_iter()
            .filter(|s| {
                !matches!(
                    s,
                    PhysStage::CleanupFreshnessRead
                        | PhysStage::CleanupRemoval
                        | PhysStage::ObligationDischarge
                )
            })
            .collect();
        format!("phys:{}", rng.choose(&stages).label())
    } else {
        let stage = *rng.choose(&OpenStage::ALL);
        let edge_tag = if rng.range(2) == 0 {
            let _ = OpenEdge::Before;
            "before"
        } else {
            let _ = OpenEdge::After;
            "after"
        };
        format!("open:{}:{edge_tag}", opening_stage_index(stage))
    }
}

pub(super) fn commit_fresh(
    state: &mut HostOracleState,
    session: &mut Session,
    kv: (u64, u64),
    step: usize,
    exec: &ExecutionRequest<'_, '_>,
) {
    let (key, value) = kv;
    let ctx = state.ctx(step, format!("commit({key:#x}={value})"), "none");
    let ref_before = state.ref_state(exec);
    let resp = session.send(Request::Mutate { key, value });
    let mut_proj = resp.mutation.expect("fresh commit mutation projection");
    assert_eq!(mut_proj.fate, MutationFate::Committed, "{ctx}: fate");
    let ref_out = requests::set(state.ref_home.runtime(), key, value);
    let ref_mut = receipts::observe(&ref_out, &ref_before.head);
    assert_eq!(ref_mut.fate, MutationFate::Committed, "{ctx}: ref fate");
    let home_state = resp.state.unwrap();
    let expected = state.ref_state(exec);
    state.compare_state(&home_state, &expected, &ctx);
    receipts::compare(
        state.map.identities_mut(),
        &mut_proj,
        &ref_mut,
        &home_state,
        &expected,
    )
    .unwrap_or_else(|e| panic!("{ctx}: live receipt correspondence mismatch: {e:?}"));
    let cad = inspect_cadence(session);
    if cad["relational.canonical-commits"].0 > 0 {
        state.derived_discarded = false;
        state.discard_gen = None;
    }
    state.committed.push((key, value, ref_mut));
}

pub(super) fn verify_retry_and_drift(
    state: &mut HostOracleState,
    session: &mut Session,
    rng: &mut SplitMix64,
    step: usize,
    exec: &ExecutionRequest<'_, '_>,
) {
    let idx = rng.range(state.committed.len());
    let (key, value) = (state.committed[idx].0, state.committed[idx].1);
    let ctx = state.ctx(step, format!("retry+drift({key:#x})"), "none");
    let before_contacts = session.send(Request::Observe).contacts;
    let retry = session.send(Request::Mutate { key, value });
    assert_eq!(retry.contacts, before_contacts, "{ctx}: retry 0 contacts");
    let retry_mut = retry.mutation.as_ref().expect("retry mutation projection");
    assert_eq!(retry_mut.fate, MutationFate::Committed, "{ctx}: retry fate");
    let retry_state = retry.state.unwrap();
    let expected = state.ref_state(exec);
    state.compare_state(&retry_state, &expected, &ctx);
    receipts::compare(
        state.map.identities_mut(),
        retry_mut,
        &state.committed[idx].2,
        &retry_state,
        &expected,
    )
    .unwrap_or_else(|e| panic!("{ctx}: retry receipt mismatch against original: {e:?}"));

    let drift_val = value ^ 0x5A5A;
    let drift = session.send(Request::Mutate {
        key,
        value: drift_val,
    });
    let drift_mut = drift.mutation.expect("drift mutation projection");
    assert_eq!(drift_mut.fate, MutationFate::IntentDrift, "{ctx}: drift");
    assert_eq!(drift.contacts, before_contacts, "{ctx}: drift 0 contacts");
    let ref_drift_out = requests::set(state.ref_home.runtime(), key, drift_val);
    let ref_drift = receipts::observe(&ref_drift_out, &expected.head);
    let drift_state = drift.state.unwrap();
    state.compare_state(&drift_state, &expected, &ctx);
    receipts::compare(
        state.map.identities_mut(),
        &drift_mut,
        &ref_drift,
        &drift_state,
        &expected,
    )
    .unwrap_or_else(|e| panic!("{ctx}: drift receipt correspondence mismatch: {e:?}"));
}

pub(super) fn run_mid_op_seam_kill(
    state: &mut HostOracleState,
    mut session: Session,
    rng: &mut SplitMix64,
    step: usize,
    ordinals: MutateSeamOrdinals,
    target_custody: bool,
    exec: &ExecutionRequest<'_, '_>,
) -> Session {
    let (key, value) = state.alloc_kv(rng);
    let (ordinal, seam_name) = if target_custody {
        (ordinals.custody_after_wal, "after_wal")
    } else {
        let choices = [
            ordinals.pre_op_after_wal,
            ordinals.custody_after_wal,
            ordinals.committed_after_wal,
        ];
        (*rng.choose(&choices), rng.choose(&ALL_SEAMS).0)
    };
    let spec = format!("mutate:{ordinal}:{seam_name}");
    if rng.range(2) == 0 {
        let pre_kill_ctx = state.ctx(
            step,
            format!("pre-seam-kill({key:#x}={value})"),
            "kill_live",
        );
        session.kill_live_with_ctx(&pre_kill_ctx.to_string());
    } else {
        let pre_close_ctx = state.ctx(
            step,
            format!("pre-seam-close({key:#x}={value})"),
            "close_normal",
        );
        close_normal(&mut session, state.discard_gen, &pre_close_ctx.to_string());
        state.derived_discarded = false;
        state.discard_gen = None;
    }

    let kill_build = rng.next_other_build(state.build);
    let kill_ctx = StepCtx {
        build: kill_build,
        ..state.ctx(step, format!("mid-op-kill({key:#x}={value})"), &spec)
    };
    kill_at_seam_with_ctx(
        &state.home_dir,
        &spec,
        key,
        value,
        kill_build,
        &kill_ctx.to_string(),
    );

    let cut_desc = if rng.range(2) == 0 {
        let rec_spec = recovery_cut_spec(rng, seam_name == "after_wal");
        let rec_build = rng.next_other_build(kill_build);
        let rec_ctx = StepCtx {
            cut: format!("{spec}+{rec_spec}"),
            build: rec_build,
            ..kill_ctx.clone()
        };
        kill_recovery_with_build_ctx(&state.home_dir, &rec_spec, rec_build, &rec_ctx.to_string());
        state.build = rng.next_other_build(rec_build);
        format!("{spec}+{rec_spec}")
    } else {
        state.build = rng.next_other_build(kill_build);
        spec
    };

    let ctx = state.ctx(step, format!("mid-op-kill({key:#x}={value})"), cut_desc);
    let ch = state.next_ch("midkill");
    let (mut s2, opened) = Session::start_with_build(&state.home_dir, &ch, 0, Some(state.build));
    assert_eq!((opened.initialized, opened.contacts), (0, 0), "{ctx}");
    let reopened = opened.state.unwrap();
    let ref_before = state.ref_state(exec);
    if target_custody {
        assert_eq!(
            reopened.custody.len(),
            1,
            "{ctx}: known custody cut MUST reopen with 1 pending custody row"
        );
    }

    if !reopened.custody.is_empty() {
        assert_eq!(reopened.custody.len(), 1, "{ctx}: 1 custody row");
        comparison::compare_head(state.map.identities_mut(), &reopened.head, &ref_before.head)
            .unwrap_or_else(|e| panic!("{ctx}: custody head divergence: {e:?}"));
        let retry_u = s2.send(Request::Mutate { key, value });
        assert_eq!(
            retry_u.mutation.unwrap().fate,
            MutationFate::ProductUnpublished,
            "{ctx}: custody retry"
        );
        assert_eq!(retry_u.contacts, 0, "{ctx}: custody retry 0 contacts");
        let cont = s2.send(Request::ContinueOriginal {
            key,
            value,
            pause_after_disposition: false,
        });
        assert_eq!(cont.contacts, 0, "{ctx}: continue 0 contacts");
        let ref_out = requests::set(state.ref_home.runtime(), key, value);
        let ref_mut = receipts::observe(&ref_out, &ref_before.head);
        let ref_after = state.ref_state(exec);
        state.compare_state(&cont.state.unwrap(), &ref_after, &ctx);
        let retry = s2.send(Request::Mutate { key, value });
        let retry_mut = retry.mutation.expect("post-cont retry receipt");
        assert_eq!(retry_mut.fate, MutationFate::Committed, "{ctx}: post-cont");
        assert_eq!(retry.contacts, 0, "{ctx}: post-cont 0 contacts");
        let retry_state = retry.state.unwrap();
        receipts::compare(
            state.map.identities_mut(),
            &retry_mut,
            &ref_mut,
            &retry_state,
            &ref_after,
        )
        .unwrap_or_else(|e| panic!("{ctx}: post-cont retry receipt mismatch: {e:?}"));
        state.committed.push((key, value, ref_mut));
    } else if reopened.head.generation == ref_before.head.generation {
        state.compare_state(&reopened, &ref_before, &ctx);
        commit_fresh(state, &mut s2, (key, value), step, exec);
    } else {
        let ref_out = requests::set(state.ref_home.runtime(), key, value);
        let ref_mut = receipts::observe(&ref_out, &ref_before.head);
        let ref_after = state.ref_state(exec);
        state.compare_state(&reopened, &ref_after, &ctx);
        let retry = s2.send(Request::Mutate { key, value });
        let retry_mut = retry.mutation.expect("committed retry receipt");
        assert_eq!(retry_mut.fate, MutationFate::Committed, "{ctx}: retry");
        assert_eq!(retry.contacts, 0, "{ctx}: committed retry 0 contacts");
        let retry_state = retry.state.unwrap();
        state.compare_state(&retry_state, &ref_after, &ctx);
        receipts::compare(
            state.map.identities_mut(),
            &retry_mut,
            &ref_mut,
            &retry_state,
            &ref_after,
        )
        .unwrap_or_else(|e| panic!("{ctx}: committed retry receipt mismatch: {e:?}"));
        state.committed.push((key, value, ref_mut));
    }

    let cad = inspect_cadence(&mut s2);
    if cad["relational.canonical-commits"].0 > 0 {
        state.derived_discarded = false;
        state.discard_gen = None;
    }
    s2
}

pub(super) fn run_checkpoint_or_close_seam_kill(
    state: &mut HostOracleState,
    mut session: Session,
    rng: &mut SplitMix64,
    step: usize,
    exec: &ExecutionRequest<'_, '_>,
) -> Session {
    let op = if rng.range(2) == 0 {
        "checkpoint"
    } else {
        "close"
    };
    let seam_name = rng.choose(&ALL_SEAMS).0;
    let spec = format!("{op}:1:{seam_name}");
    if rng.range(2) == 0 {
        let pre_kill_ctx = state.ctx(step, format!("pre-{op}-seam-kill"), "kill_live");
        session.kill_live_with_ctx(&pre_kill_ctx.to_string());
    } else {
        let pre_close_ctx = state.ctx(step, format!("pre-{op}-seam-close"), "close_normal");
        close_normal(&mut session, state.discard_gen, &pre_close_ctx.to_string());
        state.derived_discarded = false;
        state.discard_gen = None;
    }
    let kill_build = rng.next_other_build(state.build);
    let kill_ctx = StepCtx {
        build: kill_build,
        ..state.ctx(step, format!("{op}-seam-kill"), &spec)
    };
    let (key, value) = state.alloc_kv(rng);
    kill_at_seam_with_ctx(
        &state.home_dir,
        &spec,
        key,
        value,
        kill_build,
        &kill_ctx.to_string(),
    );
    state.build = rng.next_other_build(kill_build);
    let ctx = state.ctx(step, format!("{op}-seam-kill"), spec);
    let ch = state.next_ch("cpclose");
    let (mut s2, opened) = Session::start_with_build(&state.home_dir, &ch, 0, Some(state.build));
    assert_eq!((opened.initialized, opened.contacts), (0, 0), "{ctx}");
    let reopened = opened.state.unwrap();
    let ref_before = state.ref_state(exec);
    if op == "close" {
        // `close` seam kills after committing `(key, value)` and during Home close.
        let ref_out = requests::set(state.ref_home.runtime(), key, value);
        let ref_mut = receipts::observe(&ref_out, &ref_before.head);
        let ref_after = state.ref_state(exec);
        state.compare_state(&reopened, &ref_after, &ctx);
        let retry = s2.send(Request::Mutate { key, value });
        let retry_mut = retry.mutation.expect("close-seam retry receipt");
        assert_eq!(retry_mut.fate, MutationFate::Committed, "{ctx}");
        assert_eq!(retry.contacts, 0, "{ctx}: retry 0 contacts");
        let retry_state = retry.state.unwrap();
        receipts::compare(
            state.map.identities_mut(),
            &retry_mut,
            &ref_mut,
            &retry_state,
            &ref_after,
        )
        .unwrap_or_else(|e| panic!("{ctx}: close-seam retry receipt mismatch: {e:?}"));
        state.committed.push((key, value, ref_mut));
    } else {
        // `checkpoint` seam kills during checkpoint without mutating.
        state.compare_state(&reopened, &ref_before, &ctx);
    }
    let cad = inspect_cadence(&mut s2);
    if cad["relational.canonical-commits"].0 > 0 {
        state.derived_discarded = false;
        state.discard_gen = None;
    }
    s2
}
