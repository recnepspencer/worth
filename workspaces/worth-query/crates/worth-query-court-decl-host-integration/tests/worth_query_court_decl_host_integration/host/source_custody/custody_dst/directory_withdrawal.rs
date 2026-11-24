//! Real Directory Home retained-withdrawal seeds for `custody_dst`, deriving `execCount` from
//! the live handler contact counter and `durableOriginal` from `has_durable_original()`.
use super::super::*;
use super::{cancelled_scope, ObservedState, TraceLog};
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

    // Seed 60: Directory expired principal -> Retained (durable_original = true) -> cancelled
    // withdraw refuses -> fresh erased `close_unexecuted` discharges durable original with 0
    // handler contacts and unchanged balance.
    log.reset();
    {
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
                "Described",
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
        assert!(
            retained.has_durable_original(),
            "Directory Home Retained owner must observe a real durable original"
        );
        log.record(
            "ResumeRetainAndRefuse",
            "",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                "PendingAdvance",
                true,
                retained.has_durable_original(),
            ),
        );

        let (retained, denial) = retained
            .close_unexecuted(&cancelled)
            .expect_err("cancelled withdrawal must preserve durable original");
        assert!(matches!(
            denial,
            WorthQueryOperationCloseDenial::Execution(_)
        ));
        assert!(retained.has_durable_original());
        log.record(
            "WithdrawUnexecuted",
            "\"succeeds\":false",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                55,
                "PendingAdvance",
                true,
                retained.has_durable_original(),
            ),
        );

        let withdrawal = retained
            .close_unexecuted(&fresh)
            .unwrap_or_else(|(_, d)| panic!("seed 60 fresh withdrawal: {d:?}"));
        assert!(matches!(
            withdrawal,
            WorthQuerySourceCustodyWithdrawal::Closed(_)
        ));
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
                false,
            ),
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
        assert_eq!(read(&fresh, account()), Some(vec![7]));
    }

    // Seed 61: Same idempotency key commits fresh on the same Directory Home, proving the
    // withdrawn durable original was genuinely discharged on disk.
    log.reset();
    {
        let start_contacts = contacts.load(Ordering::SeqCst);
        let fresh = home.runtime().request(&principal, &live_scope);
        let described = capture(&fresh, set_balance(21), 0x4453_5460_u64);
        let erased = Custody::Described(described).erase();
        log.record(
            "AcceptOrdinary",
            "\"erased\":true,\"durableMode\":true",
            dir_obs(
                &contacts,
                start_contacts,
                &fresh,
                21,
                "Described",
                true,
                erased.has_durable_original(),
            ),
        );
        match erased.resume(home.runtime(), &fresh) {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(matches!(
                    evidence.evidence::<u64>(),
                    Some(WorthQuerySourceCompletion::Fresh { result: 21, .. })
                ));
            }
            _ => panic!("seed 61 reused key after Directory withdrawal must commit fresh"),
        }
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
                false,
            ),
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        assert_eq!(read(&fresh, account()), Some(vec![21]));
    }

    drop((principal, expired));
    close_home(home, execution);
}
