//! C3.5 owner court: concurrent unobserved attempts with later frames recover
//! monotonically across a real process kill, while corrupt/truncated frames refuse.
use super::*;
use crate::process::host::durable_retry::child_reexecution as reexec;
use std::sync::Condvar;
use worth_query_execution::facade::application_home_integration::{
    QueryDispatchFrame, QueryDispatchJournalDenial, QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES,
};
use worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_host::facade::primary_graph::{
    reconcile_recovery_handle, safe_retry_recovery_handle, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationIdempotencyBinding, WorthQueryDispatchOutboxDurabilityPosture,
    WorthQueryInboundAdmissionDenial, WorthQueryRecoveryClaimStatus as Status,
    WorthQueryRecoveryHandleDenialKind,
};
use worth_query_testkit_host::facade::host::dispatch_home::notify::BalanceValueBinding;
use worth_query_testkit_host::facade::host::dispatch_home::recoverable::{
    RecoverableNotice, RecoverableNoticeIntent,
};
use worth_query_testkit_host::facade::host::dispatch_home::schema::Account;

#[derive(serde::Serialize, serde::Deserialize)]
struct ConcurrentKillExpectation {
    parked: [([u8; 32], u64); 2],
    completed: ([u8; 32], u64),
}

struct ConcurrentParkRail {
    home: OnceLock<Weak<Home>>,
    parked_count: Mutex<Vec<([u8; 32], u64)>>,
    parked_ready: Condvar,
    never_release: Condvar,
}

impl ConcurrentParkRail {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            home: OnceLock::new(),
            parked_count: Mutex::new(Vec::new()),
            parked_ready: Condvar::new(),
            never_release: Condvar::new(),
        })
    }

    fn wait_until_parked(&self, target: usize) -> Vec<([u8; 32], u64)> {
        let mut guard = self.parked_count.lock().unwrap();
        while guard.len() < target {
            guard = self.parked_ready.wait(guard).unwrap();
        }
        guard.clone()
    }
}

impl WorthQueryExternalEffectTransport for ConcurrentParkRail {
    fn dispatch(&self, request: WorthQueryExternalDispatchRequest<'_>) -> Outcome {
        let effect = *request.correlation_token();
        let home = self
            .home
            .get()
            .and_then(Weak::upgrade)
            .expect("Home stays alive while dispatch is in flight");
        let recorded = home
            .runtime()
            .runtime()
            .durable_dispatch_custody(&effect)
            .expect("attempt must be durable before external contact");
        assert_eq!(recorded.observation(), None, "attempt has no observation yet");
        let pos = recorded.attempt().attempt_position().get();
        let mut guard = self.parked_count.lock().unwrap();
        if guard.len() < 2 {
            guard.push((effect, pos));
            self.parked_ready.notify_all();
            // Stay parked in external contact until parent OS-kills this process.
            let _unused = self.never_release.wait(guard).unwrap();
            unreachable!("writer must be killed while attempts remain unobserved");
        }
        drop(guard);
        // Third dispatch after both parked attempts completes normally.
        Outcome::Completed
    }
}

fn emit_recoverable(home: &Home, key: u64) -> ([u8; 32], u64) {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(RecoverableNoticeIntent(key))
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("recoverable emission must commit: {outcome:?}");
    };
    assert_eq!(result, key);
    let effect = *receipt.dispatch_outbox().unwrap().correlation().bytes();
    let pos = custody(home, &effect).attempt().attempt_position().get();
    (effect, pos)
}

fn admitted<R>(
    home: &Home,
    key: u64,
    body: impl for<'step> FnOnce(
        &WorthQueryAdvancementPhase<'step>,
        &WorthQueryAdmittedApplicationOperation<
            DispatchHomeSchema,
            RecoverableNotice,
            u64,
            Account,
        >,
        WorthQueryApplicationIdempotencyBinding,
        &worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            BalanceValueBinding,
        >,
    ) -> R,
) -> R {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(RecoverableNoticeIntent(key))
        .without_source()
        .idempotency(&key)
        .with_recovery_in_program(runtime, body)
        .unwrap()
}

fn writer() {
    let location = reexec::location();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = dispatch::request(
        location,
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        handlers,
        true,
    );
    let home = Arc::new(
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| reopening::opening_failed("unobserved writer", &refused)),
    );
    let rail = ConcurrentParkRail::new();
    home.runtime()
        .runtime()
        .install_external_effect_transport(rail.clone())
        .unwrap();
    rail.home.set(Arc::downgrade(&home)).ok().unwrap();

    // Park attempt 1 inside external dispatch, then start attempt 2 on the new
    // branch head so both attempts are simultaneously durable and unobserved.
    for (idx, key) in [0x9101_u64, 0x9102_u64].into_iter().enumerate() {
        let h = Arc::clone(&home);
        std::thread::spawn(move || {
            worth_execution::run_with_execution_stack(|| {
                let _ = emit_recoverable(&h, key);
            });
        });
        let _ = rail.wait_until_parked(idx + 1);
    }
    let parked_vec = rail.wait_until_parked(2);
    let parked = [parked_vec[0], parked_vec[1]];
    // Write later frames AFTER both concurrent unobserved attempts:
    // (1) a third completed dispatch (Attempt + Observation) and
    // (2) a Home-level SourceControl frame.
    let completed = emit_recoverable(&home, 0x9103);
    assert!(
        completed.1 > parked[0].1 && completed.1 > parked[1].1,
        "later completed attempt follows both unobserved attempts"
    );
    let (source_route, _, _) = super::source_controls::install(&home, "before-kill");
    home.runtime()
        .runtime()
        .retire_inbound_occurrence_source(&source_route.handle)
        .unwrap();
    let expectation = ConcurrentKillExpectation { parked, completed };
    std::fs::write(
        reexec::expected(),
        serde_json::to_vec(&expectation).unwrap(),
    )
    .unwrap();
    reexec::wait_for_kill();
}

fn reader() {
    let location = reexec::location();
    let expectation: ConcurrentKillExpectation =
        serde_json::from_slice(&std::fs::read(reexec::expected()).unwrap()).unwrap();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    // First reopen: admits both concurrent unobserved attempts + later frames,
    // recording Unresolved(UnknownProviderOutcome) before service without contact.
    let rail1 = ScriptedRail::answering(&[]);
    let home1 = open(&location, execution, &handlers, false, &rail1);
    assert!(rail1.contacts().is_empty(), "cold readmission contacts no transport");
    assert_eq!(handlers.load(Ordering::SeqCst), 0, "cold readmission replays no handler");
    let expected_unresolved = Some(Observation::Unresolved(
        QueryDispatchUnresolvedCause::UnknownProviderOutcome,
    ));
    let mut recorded_attempts = Vec::new();
    for (effect, pos) in expectation.parked {
        let c = custody(&home1, &effect);
        assert_eq!(c.attempt().attempt_position().get(), pos);
        assert_eq!(c.observation(), expected_unresolved);
        recorded_attempts.push(c);
    }
    let completed_custody = custody(&home1, &expectation.completed.0);
    assert_eq!(
        completed_custody.attempt().attempt_position().get(),
        expectation.completed.1
    );
    assert_eq!(completed_custody.observation(), Some(Observation::Completed));
    let (fresh_route, _, _) = super::source_controls::install(&home1, "after-kill");
    assert_eq!(
        super::source_controls::probe(&home1, &fresh_route),
        WorthQueryInboundAdmissionDenial::SourceRetired,
        "later source-control frame survives cold recovery"
    );
    Arc::into_inner(home1)
        .expect("reopened Home is exclusive")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("settled Home closes cleanly: {cause:?}"))
        .reclaim();

    // Second reopen (monotone repeated fold): identical attempts & observations,
    // zero extra contacts; existing recovery handles reconcile and safe-retry.
    let rail2 = ScriptedRail::answering(&[Outcome::Completed]);
    let home2 = open(&location, execution, &handlers, false, &rail2);
    assert!(rail2.contacts().is_empty());
    for (idx, (effect, _)) in expectation.parked.into_iter().enumerate() {
        assert_eq!(
            custody(&home2, &effect),
            recorded_attempts[idx],
            "repeated fold is monotone"
        );
    }
    assert_eq!(custody(&home2, &expectation.completed.0), completed_custody);

    // Unresolved recovery handle refuses completion settlement before reconciliation.
    let handle1 = admitted(&home2, 0x9101, |phase, admission, binding, input| {
        home2
            .runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap();
    let corr1 = *handle1.binding().dispatch_outbox().unwrap().correlation().bytes();
    let pos1 = expectation
        .parked
        .iter()
        .find_map(|(eff, pos)| (*eff == corr1).then_some(*pos))
        .expect("0x9101 correlation matches one of the parked attempts");
    admitted(&home2, 0x9101, |phase, admission, _, _| {
        let err = home2
            .runtime()
            .runtime()
            .settle_recorded_recovery_completion(phase, handle1, admission)
            .unwrap_err();
        assert_eq!(err.kind(), WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture);
        let (_, retained) = err.into_parts();
        let handle1 = retained.expect("live claim preserved");
        let auth = home2
            .runtime()
            .runtime()
            .admit_recovery_effect_authority(&handle1, admission)
            .unwrap();
        reconcile_recovery_handle(phase, handle1, &auth).unwrap();
    });
    assert!(rail2.contacts().is_empty(), "reconcile does not redispatch");

    // Safe retry on the second recovered attempt dispatches a fresh attempt and completes.
    let handle2 = admitted(&home2, 0x9102, |phase, admission, binding, input| {
        home2
            .runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap();
    let corr2 = *handle2.binding().dispatch_outbox().unwrap().correlation().bytes();
    assert_ne!(corr1, corr2);
    let pos2 = expectation
        .parked
        .iter()
        .find_map(|(eff, pos)| (*eff == corr2).then_some(*pos))
        .expect("0x9102 correlation matches the other parked attempt");
    admitted(&home2, 0x9102, |phase, admission, _, _| {
        let rt = home2.runtime().runtime();
        let auth = rt.admit_recovery_effect_authority(&handle2, admission).unwrap();
        let performed = rt
            .redispatch_admitted_external_effect(phase, &handle2, &auth, admission)
            .unwrap();
        let done = safe_retry_recovery_handle(phase, handle2, &auth, performed).unwrap();
        assert_eq!(
            done.outbox_durability(),
            WorthQueryDispatchOutboxDurabilityPosture::DurableHome
        );
    });
    assert_eq!(rail2.contacts().len(), 1);
    let after_reconcile = custody(&home2, &corr1);
    assert_eq!(after_reconcile.attempt().attempt_position().get(), pos1);
    assert_eq!(after_reconcile.observation(), expected_unresolved);
    let after_retry = custody(&home2, &corr2);
    assert!(
        after_retry.attempt().attempt_position().get() > pos2,
        "retry records a new attempt rather than mutating the already-observed attempt"
    );
    assert_eq!(after_retry.observation(), Some(Observation::Completed));
    assert_eq!(
        admitted(&home2, 0x9101, |phase, admission, binding, _| {
            home2
                .runtime()
                .runtime()
                .resolve_admitted_recovery_claim_status(phase, admission, binding)
                .unwrap()
                .unwrap()
        }),
        Status::Consumed
    );
    assert_eq!(
        admitted(&home2, 0x9102, |phase, admission, binding, _| {
            home2
                .runtime()
                .runtime()
                .resolve_admitted_recovery_claim_status(phase, admission, binding)
                .unwrap()
                .unwrap()
        }),
        Status::Completed
    );
    assert_eq!(handlers.load(Ordering::SeqCst), 0);
    Arc::into_inner(home2)
        .expect("exclusive")
        .close(&execution)
        .unwrap()
        .reclaim();
}

#[test]
fn concurrent_unobserved_attempts_with_later_frames_recover_monotonically_across_kill() {
    reexec::drive_test(
        "host::application_home::dispatch_custody::unobserved_recovery::concurrent_unobserved_attempts_with_later_frames_recover_monotonically_across_kill",
        writer,
        reader,
    );
}

#[test]
fn dispatch_frame_codec_still_refuses_truncated_and_corrupt_bytes() {
    assert!(matches!(
        QueryDispatchFrame::readmit(&[], QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES),
        Err(QueryDispatchJournalDenial::MalformedFrame | QueryDispatchJournalDenial::Checksum)
    ));
    let garbage = vec![0xAAu8; 128];
    assert_eq!(
        QueryDispatchFrame::readmit(&garbage, QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES),
        Err(QueryDispatchJournalDenial::Checksum)
    );
}
