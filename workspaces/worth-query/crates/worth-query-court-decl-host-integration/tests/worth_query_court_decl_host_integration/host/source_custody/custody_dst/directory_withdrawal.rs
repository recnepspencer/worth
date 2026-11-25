//! Real Directory Home retained-withdrawal seeds for `custody_dst`, deriving `execCount` from
//! the live handler contact counter and `durableOriginal` from `has_durable_original()`.
use super::super::*;
use super::{cancelled_scope, ledger_has_original, phase_label, ObservedState, TraceLog};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::{WorthQueryOperationCloseDenial, WorthQuerySourceOnlyDenial},
    WorthQueryAnySourceCustodyStep, WorthQuerySourceCustodyWithdrawal,
};

fn dir_obs(
    contacts: &AtomicUsize,
    start_contacts: usize,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    target_balance: u64,
    phase: &'static str,
    erased: bool,
    durable_original: bool,
) -> ObservedState {
    let exec_count = u8::try_from(contacts.load(Ordering::SeqCst) - start_contacts).unwrap();
    let balance = read(request, account()).unwrap()[0];
    ObservedState {
        phase,
        erased,
        durable_mode: true,
        durable_original,
        exec_count,
        committed_value: exec_count > 0 && balance == target_balance,
    }
}

pub(super) fn run_directory_withdrawal_seeds(
    clock: &WorthQueryAdmissionClock,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    log: &mut TraceLog,
) {
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open_home_with_contacts(execution, limits, contacts.clone())
        .unwrap_or_else(|refused| panic!("Directory withdrawal DST Home opens: {refused}"));
    assert!(home.is_directory());
    let live_scope = scope(clock);
    let principal = local_principal(&home, clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&live_scope)
        .unwrap();

    // One brief expired principal shared across the retained-withdrawal seed.
    let brief = Duration::from_millis(25);
    let expired = local_principal(&home, clock, "subject", brief)
        .authenticate_local(&live_scope)
        .unwrap();
    std::thread::sleep(brief + Duration::from_millis(15));

    // Seed 60: Directory expired principal -> Retained (exact durable original identity verified
    // present in Home's ledger) -> cancelled withdraw preserves ledger entry -> fresh erased
    // `close_unexecuted` discharges exact original from ledger with 0 handler contacts.
    log.reset();
    let withdrawn_original = {
        let start_contacts = contacts.load(Ordering::SeqCst);
        let fresh = home.runtime().request(&principal, &live_scope);
        let expired_req = home.runtime().request(&expired, &live_scope);
        let cancelled_sc = cancelled_scope(clock);
        let cancelled = home.runtime().request(&principal, &cancelled_sc);

        let described = capture(&expired_req, set_balance(55), 0x4453_5460_u64);
        let erased = Custody::Described(described).erase();
        assert!(!erased.has_durable_original());
        log.record(
            "AcceptOrdinary",
            "\"erased\":true,\"durableMode\":true",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                phase_label(erased.phase()),
                true,
                erased.has_durable_original(),
            ),
        );

        let retained = match erased.resume(home.runtime(), &expired_req) {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Refused(WorthQuerySourceOnlyDenial::Mutation(
                        _
                    ))
                ));
                custody
            }
            _ => panic!("seed 60 expired principal must retain durable original"),
        };
        let orig_id = retained
            .durable_original_identity()
            .expect("Directory Retained owner must carry durable original identity");
        let in_ledger_before = ledger_has_original(&home, execution, Some(orig_id));
        assert!(
            in_ledger_before && retained.has_durable_original(),
            "exact durable original identity must be live in the Home ledger before withdrawal"
        );
        log.record(
            "ResumeRetainAndRefuse",
            "",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                phase_label(retained.phase()),
                true,
                in_ledger_before,
            ),
        );

        let (retained, denial) = retained
            .close_unexecuted(&cancelled)
            .expect_err("cancelled withdrawal must preserve durable original");
        assert!(matches!(
            denial,
            WorthQueryOperationCloseDenial::Execution(_)
        ));
        assert_eq!(retained.durable_original_identity(), Some(orig_id));
        let in_ledger_after_cancel = ledger_has_original(&home, execution, Some(orig_id));
        assert!(
            in_ledger_after_cancel,
            "cancelled withdrawal must leave exact durable original live in the Home ledger"
        );
        log.record(
            "WithdrawUnexecuted",
            "\"succeeds\":false",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                phase_label(retained.phase()),
                true,
                in_ledger_after_cancel,
            ),
        );

        let withdrawal = retained
            .close_unexecuted(&fresh)
            .unwrap_or_else(|(_, d)| panic!("seed 60 fresh withdrawal: {d:?}"));
        assert!(matches!(
            withdrawal,
            WorthQuerySourceCustodyWithdrawal::Closed(_)
        ));
        let in_ledger_after_withdraw = ledger_has_original(&home, execution, Some(orig_id));
        assert!(
            !in_ledger_after_withdraw,
            "exact durable original identity must be absent from the Home ledger after close_unexecuted"
        );
        log.record(
            "WithdrawUnexecuted",
            "\"succeeds\":true",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                "TerminalWithdrawn",
                true,
                in_ledger_after_withdraw,
            ),
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
        assert_eq!(read(&fresh, account()), Some(vec![7]));
        orig_id
    };

    // Seed 61: Same idempotency key retains a new distinct durable original identity, verifies
    // the prior withdrawn original stays absent, commits fresh, and verifies the new original is
    // discharged from the Home ledger after terminal close.
    log.reset();
    {
        let start_contacts = contacts.load(Ordering::SeqCst);
        let fresh = home.runtime().request(&principal, &live_scope);
        let expired_req = home.runtime().request(&expired, &live_scope);
        assert!(!ledger_has_original(
            &home,
            execution,
            Some(withdrawn_original)
        ));
        let described = capture(&expired_req, set_balance(21), 0x4453_5460_u64);
        let erased = Custody::Described(described).erase();
        log.record(
            "AcceptOrdinary",
            "\"erased\":true,\"durableMode\":true",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                21,
                phase_label(erased.phase()),
                true,
                erased.has_durable_original(),
            ),
        );
        let retained = match erased.resume(home.runtime(), &expired_req) {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Refused(WorthQuerySourceOnlyDenial::Mutation(
                        _
                    ))
                ));
                custody
            }
            _ => panic!("seed 61 expired principal must retain new durable original"),
        };
        let second_orig = retained
            .durable_original_identity()
            .expect("seed 61 must mint a fresh durable original identity");
        assert_ne!(second_orig, withdrawn_original);
        let second_live = ledger_has_original(&home, execution, Some(second_orig));
        assert!(second_live);
        log.record(
            "ResumeRetainAndRefuse",
            "",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                21,
                phase_label(retained.phase()),
                true,
                second_live,
            ),
        );
        match retained.resume(home.runtime(), &fresh) {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 21, .. })
                ));
            }
            _ => panic!("seed 61 reused key after Directory withdrawal must commit fresh"),
        }
        let second_after_commit = ledger_has_original(&home, execution, Some(second_orig));
        assert!(
            !second_after_commit,
            "committed durable original must be discharged from the Home ledger"
        );
        log.record(
            "ResumeCommitClosed",
            "",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                21,
                "TerminalCommitted",
                true,
                second_after_commit,
            ),
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        assert_eq!(read(&fresh, account()), Some(vec![21]));
    }

    drop((principal, expired));
    close_home(home, execution);
}
