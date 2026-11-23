//! Shared adversarial MemoryOnly reference & reopen verifier for C4b courts.
use super::super::*;
use super::{build_id, BUILD_PRIMARY};
use host_interruption::opening::{AdversarialPlacement, PhysicalPauseHandle};
use host_observation::{
    adversarial_verdict::{
        classify_and_verify_reopen, verify_continued_parity, ContinuedParityCheck, RecoveredPrefix,
    },
    comparison::{self, HostOracleComparison},
    opening::PlacementOpenConfig,
    projection::{self, StateProjection},
    receipts::{self, MutationFate, MutationProjection},
    requests, OracleHome,
};
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome as Continued;

pub(super) const PREFIX_KEY: u64 = 0x4B01_0001;
pub(super) const PREFIX_VAL: u64 = 101;
pub(super) const TARGET_KEY: u64 = 0x4B01_0002;
pub(super) const TARGET_VAL: u64 = 202;
pub(super) const FOLLOW_KEY: u64 = 0x4B01_0003;
pub(super) const FOLLOW_VAL: u64 = 303;

pub(super) struct DirOpen<'a> {
    pub location: &'a Path,
    pub build: &'a str,
    pub placement: Option<AdversarialPlacement>,
    pub contacts: Arc<AtomicUsize>,
}

pub(super) fn open_dir_configured(
    cfg: DirOpen<'_>,
    execution: &ExecutionRequest<'_, '_>,
) -> (OracleHome, Option<PhysicalPauseHandle>) {
    host_observation::opening::open_with_placement(
        PlacementOpenConfig {
            location: Location::Directory(cfg.location.to_path_buf()),
            build: build_id(cfg.build),
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: cfg.contacts,
            placement: cfg.placement,
            limits: Default::default(),
        },
        execution,
    )
}

pub(super) fn open_dir(
    location: &Path,
    placement: Option<AdversarialPlacement>,
    contacts: Arc<AtomicUsize>,
    execution: &ExecutionRequest<'_, '_>,
) -> (OracleHome, Option<PhysicalPauseHandle>) {
    open_dir_configured(
        DirOpen {
            location,
            build: BUILD_PRIMARY,
            placement,
            contacts,
        },
        execution,
    )
}

pub(super) fn prepare_prefix(location: &Path, execution: &ExecutionRequest<'_, '_>) {
    let (home, _) = open_dir(location, None, Arc::new(AtomicUsize::new(0)), execution);
    assert!(requests::set(home.runtime(), PREFIX_KEY, PREFIX_VAL)
        .receipt()
        .is_some());
    home.close(execution)
        .unwrap_or_else(|e| panic!("prefix close: {e:?}"))
        .reclaim();
}

pub(super) struct ReferenceSnapshots {
    pub before: StateProjection,
    pub after: StateProjection,
    pub after_follow: StateProjection,
    pub target_receipt: MutationProjection,
    pub follow_receipt: MutationProjection,
    pub drift_receipt: MutationProjection,
    _home: OracleHome,
}

pub(super) fn build_reference(execution: &ExecutionRequest<'_, '_>) -> ReferenceSnapshots {
    let (reference, _) = host_observation::opening::open_with_placement(
        PlacementOpenConfig {
            location: Location::MemoryOnly,
            build: build_id(BUILD_PRIMARY),
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: Arc::new(AtomicUsize::new(0)),
            placement: None,
            limits: Default::default(),
        },
        execution,
    );
    assert!(requests::set(reference.runtime(), PREFIX_KEY, PREFIX_VAL)
        .receipt()
        .is_some());
    let before = projection::observe(reference.runtime(), execution);
    let target_out = requests::set(reference.runtime(), TARGET_KEY, TARGET_VAL);
    let target_receipt = receipts::observe(&target_out, &before.head);
    drop(target_out);
    let after = projection::observe(reference.runtime(), execution);
    let drift_out = requests::set(reference.runtime(), TARGET_KEY, TARGET_VAL + 77);
    let drift_receipt = receipts::observe(&drift_out, &before.head);
    drop(drift_out);
    let follow_out = requests::set(reference.runtime(), FOLLOW_KEY, FOLLOW_VAL);
    let follow_receipt = receipts::observe(&follow_out, &after.head);
    drop(follow_out);
    let after_follow = projection::observe(reference.runtime(), execution);
    ReferenceSnapshots {
        before,
        after,
        after_follow,
        target_receipt,
        follow_receipt,
        drift_receipt,
        _home: reference,
    }
}

struct VerifyCtx<'a> {
    home: &'a OracleHome,
    map: &'a mut HostOracleComparison,
    reference: &'a ReferenceSnapshots,
    contacts: &'a Arc<AtomicUsize>,
    execution: &'a ExecutionRequest<'a, 'a>,
    label: &'a str,
}

pub(super) fn verify_reopen_and_continue(
    location: &Path,
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveredPrefix {
    verify_reopen_with_build((location, BUILD_PRIMARY), reference, execution, label)
}

pub(super) fn verify_reopen_with_build(
    target: (&Path, &str),
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveredPrefix {
    let (location, reopen_build) = target;
    let contacts = Arc::new(AtomicUsize::new(0));
    let (home, _) = open_dir_configured(
        DirOpen {
            location,
            build: reopen_build,
            placement: None,
            contacts: contacts.clone(),
        },
        execution,
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 0, "{label}: open never runs handler");
    let reopened = projection::observe(home.runtime(), execution);
    let mut map = HostOracleComparison::default();
    let verdict =
        classify_and_verify_reopen(&mut map, &reopened, &reference.before, &reference.after)
            .unwrap_or_else(|e| panic!("{label}: reopen divergence: {e:?}"));

    let mut ctx = VerifyCtx {
        home: &home,
        map: &mut map,
        reference,
        contacts: &contacts,
        execution,
        label,
    };
    match &verdict {
        RecoveredPrefix::PreOperation => verify_pre_operation(&mut ctx, &reopened),
        RecoveredPrefix::UnpublishedCustody {
            attempt_owner,
            attempt,
            original_native,
            ..
        } => verify_unpublished_custody(
            &mut ctx,
            &reopened,
            (*attempt_owner, *attempt),
            original_native,
        ),
        RecoveredPrefix::CommittedInFlight => verify_committed_in_flight(&mut ctx, &reopened),
    }

    home.close(execution)
        .unwrap_or_else(|e| panic!("{label}: recovered graceful close: {e:?}"))
        .reclaim();
    verify_post_close_reopen(
        (location, reopen_build),
        &mut map,
        &reference.after_follow,
        execution,
        label,
    );
    verdict
}

fn verify_post_close_reopen(
    target: (&Path, &str),
    map: &mut HostOracleComparison,
    after_follow: &StateProjection,
    execution: &ExecutionRequest<'_, '_>,
    label: &str,
) {
    let (location, reopen_build) = target;
    let reopen_contacts = Arc::new(AtomicUsize::new(0));
    let (again, _) = open_dir_configured(
        DirOpen {
            location,
            build: reopen_build,
            placement: None,
            contacts: reopen_contacts.clone(),
        },
        execution,
    );
    assert_eq!(reopen_contacts.load(Ordering::SeqCst), 0);
    let final_reopen = projection::observe(again.runtime(), execution);
    comparison::compare(map, &final_reopen, after_follow)
        .unwrap_or_else(|e| panic!("{label}: second reopen correspondence parity: {e:?}"));
    assert_eq!(requests::balance(again.runtime()), FOLLOW_VAL, "{label}: final balance");
    again
        .close(execution)
        .unwrap_or_else(|e| panic!("{label}: final close: {e:?}"))
        .reclaim();
}

fn verify_pre_operation(ctx: &mut VerifyCtx<'_>, reopened: &StateProjection) {
    let label = ctx.label;
    assert!(matches!(
        requests::resolve_original(ctx.home.runtime(), TARGET_KEY, TARGET_VAL).unwrap(),
        Resolution::Unseen
    ));
    let first = requests::set(ctx.home.runtime(), TARGET_KEY, TARGET_VAL);
    let first_obs = receipts::observe(&first, &reopened.head);
    assert_eq!(first_obs.fate, MutationFate::Committed, "{label}");
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), 1, "{label}: executes once");
    drop(first);
    let after_state = projection::observe(ctx.home.runtime(), ctx.execution);
    receipts::compare(
        ctx.map.identities_mut(),
        &first_obs,
        &ctx.reference.target_receipt,
        &after_state,
        &ctx.reference.after,
    )
    .unwrap_or_else(|e| panic!("{label}: first receipt: {e:?}"));
    comparison::compare(ctx.map, &after_state, &ctx.reference.after)
        .unwrap_or_else(|e| panic!("{label}: after state: {e:?}"));
    let retry = requests::set(ctx.home.runtime(), TARGET_KEY, TARGET_VAL);
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), 1, "{label}: retry no second effect");
    assert_eq!(receipts::observe(&retry, &reopened.head), first_obs);
    drop(retry);
    verify_drift_and_follow(ctx, 1);
}

fn verify_unpublished_custody(
    ctx: &mut VerifyCtx<'_>,
    reopened: &StateProjection,
    attempt_id: (worth_foundational::facade::WorldOwnerCoordinate, u64),
    original_native: &worth_relational::facade::history::RelationalCommitReceipt,
) {
    let label = ctx.label;
    let retry = requests::set(ctx.home.runtime(), TARGET_KEY, TARGET_VAL);
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), 0, "{label}: custody retry 0 calls");
    assert_eq!(
        receipts::observe(&retry, &reopened.head).fate,
        MutationFate::ProductUnpublished,
        "{label}: admitted key answers recovered custody"
    );
    drop(retry);
    let Resolution::ProductUnpublished(effects) =
        requests::resolve_original(ctx.home.runtime(), TARGET_KEY, TARGET_VAL).unwrap()
    else {
        panic!("{label}: expected ProductUnpublished resolution");
    };
    let recovery = effects.into_recovery();
    loop {
        match recovery.continue_original_in_request(ctx.execution).unwrap() {
            Continued::Pending(c) => drop(c),
            Continued::Published { .. } => break,
            other => panic!("{label}: continuation unexpected: {other:?}"),
        }
    }
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), 0, "{label}: continue 0 calls");
    let Resolution::RecoveredCommitted(rec) =
        requests::resolve_original(ctx.home.runtime(), TARGET_KEY, TARGET_VAL).unwrap()
    else {
        panic!("{label}: continued key must resolve RecoveredCommitted");
    };
    let rec_obs = receipts::observe_original(&rec);
    drop(rec);
    let continued_state = projection::observe(ctx.home.runtime(), ctx.execution);
    verify_continued_parity(&ContinuedParityCheck {
        continued: &continued_state,
        after_ref: &ctx.reference.after,
        before_head: &reopened.head,
        attempt_owner: attempt_id.0,
        attempt: attempt_id.1,
        original_native,
        receipt: rec_obs.receipt.as_ref().unwrap(),
        reference_receipt: ctx.reference.target_receipt.receipt.as_ref().unwrap(),
    })
    .unwrap_or_else(|e| panic!("{label}: continued parity: {e:?}"));
    comparison::compare(ctx.map, &continued_state, &ctx.reference.after)
        .unwrap_or_else(|e| panic!("{label}: continued correspondence: {e:?}"));
    verify_drift_and_follow(ctx, 0);
}

fn verify_committed_in_flight(ctx: &mut VerifyCtx<'_>, reopened: &StateProjection) {
    let label = ctx.label;
    let retry = requests::set(ctx.home.runtime(), TARGET_KEY, TARGET_VAL);
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), 0, "{label}: committed retry 0 calls");
    let retry_obs = receipts::observe(&retry, &ctx.reference.before.head);
    assert_eq!(retry_obs.fate, MutationFate::Committed, "{label}");
    drop(retry);
    receipts::compare(
        ctx.map.identities_mut(),
        &retry_obs,
        &ctx.reference.target_receipt,
        reopened,
        &ctx.reference.after,
    )
    .unwrap_or_else(|e| panic!("{label}: recovered receipt: {e:?}"));
    verify_drift_and_follow(ctx, 0);
}

fn verify_drift_and_follow(ctx: &mut VerifyCtx<'_>, expected_before_follow: usize) {
    let label = ctx.label;
    let drift = requests::set(ctx.home.runtime(), TARGET_KEY, TARGET_VAL + 77);
    let drift_obs = receipts::observe(&drift, &ctx.reference.before.head);
    assert_eq!(drift_obs.fate, MutationFate::IntentDrift, "{label}");
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), expected_before_follow);
    drop(drift);
    let after_target = projection::observe(ctx.home.runtime(), ctx.execution);
    receipts::compare(
        ctx.map.identities_mut(),
        &drift_obs,
        &ctx.reference.drift_receipt,
        &after_target,
        &ctx.reference.after,
    )
    .unwrap_or_else(|e| panic!("{label}: drift receipt: {e:?}"));

    let follow = requests::set(ctx.home.runtime(), FOLLOW_KEY, FOLLOW_VAL);
    let follow_obs = receipts::observe(&follow, &after_target.head);
    assert_eq!(follow_obs.fate, MutationFate::Committed, "{label}");
    assert_eq!(ctx.contacts.load(Ordering::SeqCst), expected_before_follow + 1);
    drop(follow);
    let final_state = projection::observe(ctx.home.runtime(), ctx.execution);
    receipts::compare(
        ctx.map.identities_mut(),
        &follow_obs,
        &ctx.reference.follow_receipt,
        &final_state,
        &ctx.reference.after_follow,
    )
    .unwrap_or_else(|e| panic!("{label}: follow receipt / identity non-reissue: {e:?}"));
    comparison::compare(ctx.map, &final_state, &ctx.reference.after_follow)
        .unwrap_or_else(|e| panic!("{label}: final state parity: {e:?}"));
}
