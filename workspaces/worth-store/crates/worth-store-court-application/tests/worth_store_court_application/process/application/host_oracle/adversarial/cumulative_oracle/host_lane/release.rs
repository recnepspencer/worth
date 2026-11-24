//! Post-cumulative Custody Release + ObserveReleased + kill/reopen + follow-up
//! commit verification against the live `MemoryOnly` reference.
use super::super::super::super::{
    protocol::{Disposition, Request},
    session::Session,
};
use super::super::super::dimensions_matrix::verify::close_normal;
use super::super::super::recovery_fixtures::MutateSeamOrdinals;
use super::super::super::recovery_interruption_oracle::kill_recovery_with_build_and_ctx;
use super::super::super::seam_kill_sweep::kill_at_seam_with_ctx;
use super::super::{SplitMix64, StepCtx};
use super::HostOracleState;
use worth_execution::ExecutionRequest;
use worth_store_testkit_application_oracle::facade::host_observation::{
    comparison,
    projection::{OccurrenceProvenance, StateProjection},
    receipts::{self, MutationFate},
    requests,
};

pub(super) fn kill_recovery_with_build_ctx(
    location: &std::path::Path,
    spec: &str,
    build: &str,
    ctx: &str,
) {
    kill_recovery_with_build_and_ctx(location, spec, None, Some(build), ctx);
}

pub(super) fn verify_custody_release_tail(
    state: &mut HostOracleState,
    rng: &mut SplitMix64,
    ordinals: MutateSeamOrdinals,
    ref_before: &StateProjection,
    exec: &ExecutionRequest<'_, '_>,
) {
    let (key, value) = state.alloc_kv(rng);
    let kill_build = rng.next_other_build(state.build);
    let spec = format!("mutate:{}:after_wal", ordinals.custody_after_wal);
    let kill_ctx = StepCtx {
        seed: state.seed,
        step: 100,
        family: "core-host",
        op: format!("custody-release({key:#x}={value})"),
        cut: spec.clone(),
        build: kill_build,
        derived: if state.derived_discarded {
            "d14-discarded"
        } else {
            "retained"
        },
    };
    kill_at_seam_with_ctx(
        &state.home_dir,
        &spec,
        key,
        value,
        kill_build,
        &kill_ctx.to_string(),
    );
    state.build = rng.next_other_build(kill_build);
    let ctx = state.ctx(100, format!("custody-release({key:#x}={value})"), spec);
    let ch1 = state.next_ch("rel1");
    let (mut s1, o1) = Session::start_with_build(&state.home_dir, &ch1, 0, Some(state.build));
    assert_eq!((o1.initialized, o1.contacts), (0, 0), "{ctx}: reopen");
    let st1 = o1.state.unwrap();
    assert_eq!(st1.custody.len(), 1, "{ctx}: 1 custody row");
    let custody_row = st1.custody[0].clone();
    let expected_head = custody_row
        .expected
        .as_ref()
        .unwrap_or_else(|| panic!("{ctx}: pre-release custody must report expected head"));
    let orig_eff = custody_row
        .original_effect
        .as_ref()
        .unwrap_or_else(|| panic!("{ctx}: pre-release custody must report original_effect"));
    comparison::compare_head(state.map.identities_mut(), &st1.head, &ref_before.head)
        .unwrap_or_else(|e| panic!("{ctx}: head mismatch: {e:?}"));
    let rel = s1.send(Request::ReleaseOriginal {
        key,
        value,
        expect_continued: false,
    });
    assert!(
        matches!(rel.disposition, Some(Disposition::Released(1))),
        "{ctx}: release"
    );
    let Some(Disposition::ReleasedOriginal(rel1)) =
        s1.send(Request::ObserveReleased { key, value }).disposition
    else {
        panic!("{ctx}: released original must be observable");
    };
    assert!(rel1.native_position > 0, "{ctx}: native_position > 0");
    assert!(
        rel1.release_position.is_some() && rel1.release_digest.is_some(),
        "{ctx}: Directory release must persist release position and digest"
    );
    assert_eq!(
        rel1.identity, custody_row.identity,
        "{ctx}: released identity must match pre-release custody"
    );
    assert_eq!(
        &rel1.expected, expected_head,
        "{ctx}: released expected head must match pre-release custody"
    );
    assert_eq!(
        rel1.attempt, orig_eff.attempt,
        "{ctx}: released attempt must match pre-release custody original attempt"
    );
    let assoc = rel1
        .association
        .as_ref()
        .unwrap_or_else(|| panic!("{ctx}: Directory released original must carry association"));
    assert_eq!(
        assoc.owner, orig_eff.attempt_owner,
        "{ctx}: released assoc owner must match pre-release original attempt_owner"
    );
    assert_eq!(
        assoc.attempt, orig_eff.attempt,
        "{ctx}: released assoc attempt must match pre-release original attempt"
    );
    let live_kill_ctx = state.ctx(
        100,
        format!("custody-release-live-kill({key:#x})"),
        "kill_live",
    );
    s1.kill_live_with_ctx(&live_kill_ctx.to_string());

    state.build = rng.next_other_build(state.build);
    let ctx2 = state.ctx(101, format!("custody-release-postkill({key:#x})"), "none");
    let ch2 = state.next_ch("rel2");
    let (mut s2, o2) = Session::start_with_build(&state.home_dir, &ch2, 0, Some(state.build));
    assert_eq!((o2.initialized, o2.contacts), (0, 0), "{ctx2}: reopen#2");
    assert!(o2.state.unwrap().custody.is_empty(), "{ctx2}: 0 custody");
    let Some(Disposition::ReleasedOriginal(rel2)) =
        s2.send(Request::ObserveReleased { key, value }).disposition
    else {
        panic!("{ctx2}: released original must survive kill+reopen");
    };
    assert_eq!(
        rel2, rel1,
        "{ctx2}: released projection invariant across kill"
    );

    let (fk, fv) = state.alloc_kv(rng);
    let follow = s2.send(Request::Mutate { key: fk, value: fv });
    let follow_mut = follow.mutation.expect("follow-up mutation projection");
    assert_eq!(
        follow_mut.fate,
        MutationFate::Committed,
        "{ctx2}: follow-up"
    );
    let follow_state = follow.state.unwrap();
    let ref_follow_out = requests::set(state.ref_home.runtime(), fk, fv);
    let ref_follow_mut = receipts::observe(&ref_follow_out, &ref_before.head);
    let ref_after = state.ref_state(exec);
    assert!(
        follow_state.custody.is_empty(),
        "{ctx2}: follow-up 0 custody"
    );
    fn entity_rows(
        s: &StateProjection,
    ) -> Vec<(
        &str,
        worth_relational::facade::storage::RecordLifecycleState,
        &serde_json::Value,
    )> {
        s.entities
            .iter()
            .map(|e| {
                (
                    e.kind_name.as_str(),
                    e.lifecycle,
                    &e.authoritative_aspect_state,
                )
            })
            .collect()
    }
    fn rel_rows(
        s: &StateProjection,
    ) -> Vec<(
        &str,
        worth_relational::facade::storage::RecordLifecycleState,
        &serde_json::Value,
    )> {
        s.relations
            .iter()
            .map(|r| {
                (
                    r.kind_name.as_str(),
                    r.lifecycle,
                    &r.authoritative_aspect_state,
                )
            })
            .collect()
    }
    assert_eq!(
        entity_rows(&follow_state),
        entity_rows(&ref_after),
        "{ctx2}: follow-up entity rows parity"
    );
    assert_eq!(
        rel_rows(&follow_state),
        rel_rows(&ref_after),
        "{ctx2}: follow-up relation rows parity"
    );
    receipts::compare(
        state.map.identities_mut(),
        &follow_mut,
        &ref_follow_mut,
        &follow_state,
        &ref_after,
    )
    .unwrap_or_else(|e| panic!("{ctx2}: follow-up receipt correspondence mismatch: {e:?}"));
    let hr = follow_mut.receipt.as_ref().expect("follow receipt");
    assert_eq!(hr.after, follow_state.head, "{ctx2}: follow receipt head");
    let occ = follow_state
        .occurrences
        .iter()
        .find(|o| o.commit == hr.after.commit)
        .unwrap_or_else(|| panic!("{ctx2}: follow-up occurrence #{} missing", hr.after.commit));
    assert_eq!(
        occ.provenance,
        OccurrenceProvenance::Publication(hr.attempt),
        "{ctx2}: follow-up occurrence attempt binding"
    );
    assert_eq!(
        occ.relational_publication.as_ref().map(|p| p.commit_id()),
        Some(hr.native.commit_id),
        "{ctx2}: follow-up occurrence native commit binding"
    );
    assert_eq!(
        occ.relational_publication.as_ref().map(|p| p.version_id()),
        Some(hr.native.version_id),
        "{ctx2}: follow-up occurrence native version binding"
    );
    close_normal(&mut s2, None, &format!("{ctx2}/close"));
}
