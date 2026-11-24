//! Type-erased Host custody stores commands behind one move-only slot while
//! keeping exact pending phases, receipts, and typed downcast evidence intact.
use super::*;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQuerySourceOnlyDenial},
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
};

#[test]
fn erased_source_custody_survives_interrupted_frames_and_recovers_typed_evidence() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .unwrap();
    let home = open_home(&execution, limits)
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
