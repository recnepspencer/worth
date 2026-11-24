//! Type-erased Host custody stores commands behind one move-only slot while
//! keeping exact pending phases, receipts, lawful unexecuted withdrawal, and
//! typed downcast evidence intact.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryOperationCloseDenial,
        WorthQuerySourceOnlyAdvance, WorthQuerySourceOnlyDenial,
    },
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep, WorthQuerySourceCustodyWithdrawal,
};

#[test]
fn erased_source_custody_survives_interrupted_frames_and_recovers_typed_evidence() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open_home_with_contacts(&execution, limits, contacts.clone())
        .unwrap_or_else(|refused| panic!("bounded Home opens: {refused:?}"));
    let scope = scope(&clock);
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();

    let mut pending_slot: Option<WorthQueryAnySourceCustody<Schema, fixture::HomeProgram>>;

    // 1. Cancelled before durable retention: erased custody retains Described phase.
    {
        let cancellation = WorthQueryCancellationSource::new();
        let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
        let request = home.runtime().request(&principal, &cancelled);
        let described = capture(&request, set_balance(11), 0x4552_4131_u64);
        cancellation.cancel();
        let step = Custody::Described(described)
            .erase()
            .resume(home.runtime(), &request);
        match step {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Retention(_)
                ));
                pending_slot = Some(custody);
            }
            _ => panic!("cancelled retention must stay pending"),
        }
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
    }

    // 2. Fresh frame resumes from the single erased slot and commits with typed evidence.
    {
        let request = home.runtime().request(&principal, &scope);
        let custody = pending_slot.take().expect("pending slot owns the command");
        let step = custody.resume(home.runtime(), &request);
        match step {
            WorthQueryAnySourceCustodyStep::Committed {
                closed: _,
                evidence,
            } => {
                assert!(evidence.live_receipt().is_some());
                assert!(evidence.recorded_receipt().is_none());
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 11, .. })
                ));
                let evidence = evidence
                    .into_evidence::<String>()
                    .err()
                    .expect("wrong downcast type must preserve owned completion");
                let recovered = evidence
                    .into_evidence::<u64>()
                    .unwrap_or_else(|_| panic!("exact result type downcasts owned completion"));
                assert!(matches!(
                    recovered,
                    WorthQuerySourceCompletion::Fresh { result: 11, .. }
                ));
            }
            _ => panic!("fresh frame must commit erased custody"),
        }
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
    }

    // 3. Expired principal refuses advance; fresh frame finishes via step.erase().
    {
        let brief = Duration::from_millis(40);
        let expiring = local_principal(&home, &clock, "subject", brief)
            .authenticate_local(&scope)
            .unwrap();
        std::thread::sleep(brief + Duration::from_millis(20));
        let request = home.runtime().request(&expiring, &scope);
        let described = capture(&request, set_balance(12), 0x4552_4132_u64);
        let step = Custody::Described(described)
            .resume(home.runtime(), &request)
            .erase();
        match step {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Refused(WorthQuerySourceOnlyDenial::Mutation(
                        _
                    ))
                ));
                pending_slot = Some(custody);
            }
            _ => panic!("expired principal must remain pending"),
        }
    }
    {
        let request = home.runtime().request(&principal, &scope);
        let step = pending_slot
            .take()
            .expect("pending slot owns retained command")
            .resume(home.runtime(), &request);
        match step {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 12, .. })
                ));
            }
            _ => panic!("fresh frame must commit retained erased custody"),
        }
        assert_eq!(read(&request, account()), Some(vec![12]));
        assert_eq!(contacts.load(Ordering::SeqCst), 2);
    }

    // 4. Drifted idempotency key declines and exposes typed terminal evidence.
    {
        let request = home.runtime().request(&principal, &scope);
        let drifted = capture(&request, set_balance(99), 0x4552_4131_u64);
        let step = Custody::Described(drifted)
            .erase()
            .resume(home.runtime(), &request);
        match step {
            WorthQueryAnySourceCustodyStep::Declined { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<Schema, Balance>(),
                    Some(WorthQueryTerminalSourceEvidence::Outcome(
                        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                    ))
                ));
                let owned = evidence
                    .into_evidence::<Schema, Balance>()
                    .expect("typed terminal evidence downcasts cleanly");
                assert!(matches!(
                    owned,
                    WorthQueryTerminalSourceEvidence::Outcome(
                        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
                    )
                ));
            }
            _ => panic!("drifted idempotency key must decline"),
        }
        assert_eq!(read(&request, account()), Some(vec![12]));
        assert_eq!(contacts.load(Ordering::SeqCst), 2);
    }

    // 5. Expired principal retains durable original -> erased `close_unexecuted` withdraws
    //    without running the handler and cleanly discharges the durable original.
    {
        let brief = Duration::from_millis(40);
        let expiring = local_principal(&home, &clock, "subject", brief)
            .authenticate_local(&scope)
            .unwrap();
        std::thread::sleep(brief + Duration::from_millis(20));
        let request = home.runtime().request(&expiring, &scope);
        let described = capture(&request, set_balance(77), 0x4552_4135_u64);
        let step = Custody::Described(described)
            .erase()
            .resume(home.runtime(), &request);
        let retained_erased = match step {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Refused(WorthQuerySourceOnlyDenial::Mutation(
                        _
                    ))
                ));
                custody
            }
            _ => panic!("expired principal must leave Retained unexecuted custody"),
        };
        assert_eq!(contacts.load(Ordering::SeqCst), 2);

        // Consuming typed downcast recovers the exact Retained owner; re-erasing keeps it intact.
        let typed = retained_erased
            .try_into_typed::<Balance, fixture::BalanceAction>()
            .unwrap_or_else(|_| panic!("exact type must downcast Retained custody"));
        assert!(matches!(typed, Custody::Retained(_)));
        let retained_erased = typed.erase();

        // A cancelled request refuses withdrawal without losing the erased owner.
        let cancellation = WorthQueryCancellationSource::new();
        let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
        cancellation.cancel();
        let cancelled_request = home.runtime().request(&principal, &cancelled);
        let (retained_erased, denial) = retained_erased
            .close_unexecuted(&cancelled_request)
            .expect_err("cancelled withdrawal must preserve erased custody");
        assert!(matches!(
            denial,
            WorthQueryOperationCloseDenial::Execution(_)
        ));

        // Fresh request lawfully closes the unexecuted command and discharges its durable original.
        let fresh_request = home.runtime().request(&principal, &scope);
        let withdrawal = retained_erased
            .close_unexecuted(&fresh_request)
            .unwrap_or_else(|(_, denial)| panic!("unexecuted withdrawal must succeed: {denial:?}"));
        assert!(matches!(
            withdrawal,
            WorthQuerySourceCustodyWithdrawal::Closed(_)
        ));
        assert_eq!(read(&fresh_request, account()), Some(vec![12]));
        assert_eq!(
            contacts.load(Ordering::SeqCst),
            2,
            "withdrawing an unexecuted command must never contact the handler"
        );

        // Reusing the same idempotency key with a different value commits cleanly, proving
        // the prior durable original was discharged and never committed.
        let replacement = capture(&fresh_request, set_balance(13), 0x4552_4135_u64);
        match Custody::Described(replacement)
            .erase()
            .resume(home.runtime(), &fresh_request)
        {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 13, .. })
                ));
            }
            _ => panic!("fresh command after unexecuted withdrawal must commit"),
        }
        assert_eq!(read(&fresh_request, account()), Some(vec![13]));
        assert_eq!(contacts.load(Ordering::SeqCst), 3);
    }

    // 6. Described withdrawal returns BeforeConstruction; post-execution Completed custody
    //    refuses `close_unexecuted` with NativeObligationRetained and stays resumable.
    {
        let fresh_request = home.runtime().request(&principal, &scope);
        let described = capture(&fresh_request, set_balance(88), 0x4552_4136_u64);
        let withdrawal = Custody::Described(described)
            .erase()
            .close_unexecuted(&fresh_request)
            .unwrap_or_else(|(_, denial)| panic!("described withdrawal succeeds: {denial:?}"));
        assert!(matches!(
            withdrawal,
            WorthQuerySourceCustodyWithdrawal::BeforeConstruction
        ));
        assert_eq!(contacts.load(Ordering::SeqCst), 3);

        let described = capture(&fresh_request, set_balance(14), 0x4552_4137_u64);
        let retained = described
            .retain_durable_operation::<fixture::HomeProgram, fixture::BalanceAction>(
                &fresh_request,
                home.runtime(),
            )
            .unwrap_or_else(|(_, denial)| panic!("retain succeeds: {denial:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(completed) = retained.advance(&fresh_request)
        else {
            panic!("direct advance must reach Completed");
        };
        let erased_completed = Custody::Completed(completed).erase();
        let (erased_completed, denial) = erased_completed
            .close_unexecuted(&fresh_request)
            .expect_err("post-execution Completed custody must refuse unexecuted withdrawal");
        assert!(matches!(
            denial,
            WorthQueryOperationCloseDenial::NativeObligationRetained
        ));
        match erased_completed.resume(home.runtime(), &fresh_request) {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 14, .. })
                ));
            }
            _ => panic!("preserved Completed custody must still close cleanly on resume"),
        }
        assert_eq!(read(&fresh_request, account()), Some(vec![14]));
        assert_eq!(contacts.load(Ordering::SeqCst), 4);
    }

    drop(principal);
    let closed = home
        .close_with_policy(ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        ))
        .unwrap_or_else(|refused| panic!("policy close: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}
