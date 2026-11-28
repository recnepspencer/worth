//! External Dispatch Custody (`NotifyAccountBalance`) child phases verifying
//! recovered prefixes & original-key typed fates BEFORE any settlement/repair,
//! full `ReceiptId` parity, D14 discard (`covered == 0`), and `MemoryOnly` parity.
use super::super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::super::opening;
use super::super::super::dimensions_matrix::assert_cadence_discarded;
use super::ops::{
    cadence_map, custody_count, dispatch_count, notify_dispatch, open_dispatch,
    settle_custody_if_any, ReceiptId,
};
use super::{DISPATCH_BAL, DISPATCH_KEY};
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome,
    primary_graph::WorthQueryApplicationUncommitted, WorthQueryApplicationHomeLocation as Location,
};

pub(super) fn run_dispatch_phase(
    phase: &str,
    loc: PathBuf,
    markers: &Path,
    build: &str,
    extra: &Path,
) {
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    match phase {
        "seed" => {
            let h = open_dispatch(Location::Directory(loc), build, true, None, None, &exec);
            h.close(&exec).unwrap().reclaim();
        }
        "reopen_discard_kill" => {
            let before_count = dispatch_count(extra);
            assert!(before_count <= 1, "zero duplicate dispatch across crash");
            let h = open_dispatch(
                Location::Directory(loc),
                build,
                false,
                None,
                Some(extra.to_path_buf()),
                &exec,
            );
            let cust = custody_count(&h);
            let receipt = if cust > 0 {
                assert_eq!(cust, 1, "dispatch single custody row");
                let unpub = notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL);
                assert!(
                    matches!(
                        unpub,
                        Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(_))
                    ),
                    "dispatch retry during custody must report ProductUnpublished: {unpub:?}"
                );
                assert_eq!(
                    dispatch_count(extra),
                    before_count,
                    "custody retry must not re-dispatch"
                );
                settle_custody_if_any(&h, &exec);
                let Outcome::RecoveredCommitted(r) =
                    notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL)
                else {
                    panic!("dispatch post-custody retry must return RecoveredCommitted");
                };
                ReceiptId::from_recovered(&r)
            } else if before_count == 0 {
                let Outcome::Committed { receipt, .. } =
                    notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL)
                else {
                    panic!("dispatch pre-op must commit freshly");
                };
                ReceiptId::from_live(&receipt)
            } else {
                match notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL) {
                    Outcome::RecoveredCommitted(r) => ReceiptId::from_recovered(&r),
                    Outcome::AlreadyCommitted(r) => ReceiptId::from_live(&r),
                    other => panic!("dispatch committed retry must be idempotent: {other:?}"),
                }
            };
            assert_eq!(
                dispatch_count(extra),
                1,
                "exact 1 total dispatch after settlement"
            );
            std::fs::write(
                extra.with_extension("rcpt"),
                serde_json::to_vec(&receipt).unwrap(),
            )
            .unwrap();
            let _ = checkpoint_and_discard_signal(&h, &exec);
            crate::killed_child::announce(&markers.join("ready"), b"disp-discard");
            crate::killed_child::park();
        }
        "final_verify" => {
            let expected: ReceiptId =
                serde_json::from_slice(&std::fs::read(extra.with_extension("rcpt")).unwrap())
                    .unwrap();
            let h = open_dispatch(
                Location::Directory(loc),
                build,
                false,
                None,
                Some(extra.to_path_buf()),
                &exec,
            );
            assert_cadence_discarded(&cadence_map(&h, &exec), "c4b4/dispatch/final");
            let retry_id = match notify_dispatch(&h, DISPATCH_KEY, DISPATCH_BAL) {
                Outcome::RecoveredCommitted(r) => ReceiptId::from_recovered(&r),
                Outcome::AlreadyCommitted(r) => ReceiptId::from_live(&r),
                other => panic!("dispatch post-D14 retry must be idempotent: {other:?}"),
            };
            assert_eq!(retry_id, expected, "dispatch retry receipt parity");
            assert_eq!(
                dispatch_count(extra),
                1,
                "retry across D14+reopen must never re-dispatch"
            );
            assert!(matches!(
                notify_dispatch(&h, DISPATCH_KEY + 1, DISPATCH_BAL + 1),
                Outcome::Committed { .. }
            ));
            assert_eq!(dispatch_count(extra), 2, "follow-up dispatches cleanly");
            h.close(&exec).unwrap().reclaim();
        }
        other => panic!("unknown dispatch phase {other}"),
    }
}
