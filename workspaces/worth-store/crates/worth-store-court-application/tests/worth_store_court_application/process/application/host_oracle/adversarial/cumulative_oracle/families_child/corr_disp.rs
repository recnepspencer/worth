//! Authored Correction (`reverse(&source)`) and External Dispatch Custody
//! (`NotifyAccountBalance`) child phases verifying recovered prefixes &
//! original-key typed fates BEFORE any settlement/repair, full `ReceiptId`
//! parity, D14 discard (`covered == 0`), and `MemoryOnly` parity.
use super::super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::super::opening;
use super::super::super::correction_fixture::open_label_home_with_build;
use super::super::super::dimensions_matrix::assert_cadence_discarded;
use super::super::super::BUILD_PRIMARY;
use super::ops::{
    cadence_map, corr_summary, custody_count, dispatch_count, notify_dispatch, open_dispatch,
    reverse_attempt, seed_label_edit, settle_custody_if_any, ReceiptId, ReverseFate,
};
use super::{DISPATCH_BAL, DISPATCH_KEY, REVERSE_KEY};
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome,
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryCorrectionSourceCoordinates},
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;

#[derive(serde::Serialize, serde::Deserialize)]
struct CorrRecord {
    used_key: String,
    original_discharged: bool,
    receipt: ReceiptId,
}

fn corr_ref_summaries(exec: ExecutionRequest<'_, '_>) -> ((String, String), (String, String)) {
    let ref_h = open_label_home_with_build(Location::MemoryOnly, BUILD_PRIMARY, None, exec);
    let ref_coords = seed_label_edit(&ref_h);
    let before = corr_summary(&ref_h);
    assert!(matches!(
        reverse_attempt(&ref_h, &ref_coords, REVERSE_KEY),
        ReverseFate::Committed(_)
    ));
    let after = corr_summary(&ref_h);
    drop(ref_h.close(&exec).err().unwrap().resume().unwrap());
    (before, after)
}

pub(super) fn run_correction_phase(
    phase: &str,
    loc: PathBuf,
    markers: &Path,
    build: &str,
    extra: &Path,
) {
    let lease = checkpoint_capture::lease(256 << 20);
    let exec = ExecutionRequest::leased(&lease);
    match phase {
        "seed" => {
            let h = open_label_home_with_build(Location::Directory(loc), build, None, exec);
            let coords = seed_label_edit(&h);
            std::fs::write(extra, serde_json::to_vec(&coords).unwrap()).unwrap();
            h.close(&exec).unwrap().reclaim();
        }
        "reopen_discard_kill" => {
            let (before_sum, after_sum) = corr_ref_summaries(exec);
            let coords: WorthQueryCorrectionSourceCoordinates =
                serde_json::from_slice(&std::fs::read(extra).unwrap()).unwrap();
            let h = open_label_home_with_build(Location::Directory(loc), build, None, exec);
            let observed = corr_summary(&h);
            let cust = custody_count(&h);
            let record = if cust > 0 {
                assert_eq!(cust, 1, "correction single custody row");
                assert_eq!(observed, before_sum, "correction custody keeps predecessor");
                assert!(
                    matches!(
                        reverse_attempt(&h, &coords, REVERSE_KEY),
                        ReverseFate::ProductUnpublished
                    ),
                    "correction retry of original key while custody pending must report ProductUnpublished"
                );
                assert_eq!(
                    corr_summary(&h),
                    before_sum,
                    "pending custody retry must not advance graph before settlement"
                );
                settle_custody_if_any(&h, &exec);
                assert_eq!(corr_summary(&h), after_sum, "custody settles to reversed");
                let ReverseFate::RecoveredCommitted(receipt) =
                    reverse_attempt(&h, &coords, REVERSE_KEY)
                else {
                    panic!("correction post-custody retry of original key must return RecoveredCommitted");
                };
                CorrRecord {
                    used_key: REVERSE_KEY.to_owned(),
                    original_discharged: false,
                    receipt,
                }
            } else if observed == before_sum {
                match reverse_attempt(&h, &coords, REVERSE_KEY) {
                    ReverseFate::Committed(receipt) => {
                        assert_eq!(corr_summary(&h), after_sum);
                        CorrRecord {
                            used_key: REVERSE_KEY.to_owned(),
                            original_discharged: false,
                            receipt,
                        }
                    }
                    ReverseFate::PreparationDischarged => {
                        let fresh_key = format!("{REVERSE_KEY}-fresh");
                        let ReverseFate::Committed(receipt) =
                            reverse_attempt(&h, &coords, &fresh_key)
                        else {
                            panic!(
                                "fresh correction reversal must commit after PreparationDischarged"
                            );
                        };
                        assert_eq!(corr_summary(&h), after_sum);
                        CorrRecord {
                            used_key: fresh_key,
                            original_discharged: true,
                            receipt,
                        }
                    }
                    _ => panic!("correction pre-op must commit or report PreparationDischarged"),
                }
            } else {
                assert_eq!(
                    observed, after_sum,
                    "correction committed must match MemoryOnly"
                );
                let receipt = match reverse_attempt(&h, &coords, REVERSE_KEY) {
                    ReverseFate::RecoveredCommitted(r) | ReverseFate::AlreadyCommitted(r) => r,
                    _ => panic!("correction committed retry of original key must be idempotent"),
                };
                CorrRecord {
                    used_key: REVERSE_KEY.to_owned(),
                    original_discharged: false,
                    receipt,
                }
            };
            std::fs::write(
                extra.with_extension("rcpt"),
                serde_json::to_vec(&record).unwrap(),
            )
            .unwrap();
            let _ = checkpoint_and_discard_signal(&h, &exec);
            crate::killed_child::announce(&markers.join("ready"), b"corr-discard");
            crate::killed_child::park();
        }
        "final_verify" => {
            let (_, after_sum) = corr_ref_summaries(exec);
            let coords: WorthQueryCorrectionSourceCoordinates =
                serde_json::from_slice(&std::fs::read(extra).unwrap()).unwrap();
            let record: CorrRecord =
                serde_json::from_slice(&std::fs::read(extra.with_extension("rcpt")).unwrap())
                    .unwrap();
            let h = open_label_home_with_build(Location::Directory(loc), build, None, exec);
            assert_cadence_discarded(&cadence_map(&h, &exec), "c4b4/correction/final");
            assert_eq!(corr_summary(&h), after_sum, "post-D14 summary parity");
            if record.original_discharged {
                assert!(
                    matches!(
                        reverse_attempt(&h, &coords, REVERSE_KEY),
                        ReverseFate::PreparationDischarged | ReverseFate::SourceAlreadyReversed
                    ),
                    "discharged original key must never replay as committed after D14+reopen"
                );
            }
            let retry_id = match reverse_attempt(&h, &coords, &record.used_key) {
                ReverseFate::AlreadyCommitted(r) | ReverseFate::RecoveredCommitted(r) => r,
                _ => panic!("correction retry after D14+reopen must be idempotent"),
            };
            assert_eq!(
                retry_id, record.receipt,
                "correction retry receipt must match original Native & World coordinates"
            );
            h.close(&exec).unwrap().reclaim();
        }
        other => panic!("unknown correction phase {other}"),
    }
}

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
