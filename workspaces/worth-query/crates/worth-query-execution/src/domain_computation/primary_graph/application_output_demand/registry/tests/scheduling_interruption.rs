//! Scheduling interruption preserves registry custody, not output currentness.
use super::super::{
    DemandAdmissionKind, OutputRefreshPredecessor, PreparedReadyBacking,
    WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
    WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandAdvanceAdmission, WorthQueryOutputProgress,
    WorthQueryPerformedOutputDemandSource,
};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandRecoveryPosture as Posture,
    WorthQueryOutputReadinessDeliveryEvidence,
};
use std::sync::atomic::Ordering;

mod bridge_deferral;
mod fixture;
use fixture::Fixture;

#[derive(Clone, Copy, Debug)]
enum Finish {
    Ordinary,
    Selected,
}

impl Finish {
    fn stop(
        self,
        fixture: &Fixture,
        kind: WorthQueryOutputDemandDenialKind,
    ) -> WorthQueryOutputDemandDenial {
        let mut result = Err(WorthQueryOutputDemandDenial::new(
            kind,
            "scheduling stopped",
        ));
        match self {
            Self::Ordinary => {
                let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
                    fixture.registry.begin(&fixture.interest)
                else {
                    panic!("the ordinary caller must claim scheduling");
                };
                fixture
                    .registry
                    .finish_scheduling(&fixture.interest, source, &mut result);
            }
            Self::Selected => {
                let (admission, finish) = fixture
                    .registry
                    .begin_admitted(&fixture.interest, &mut record_admission())
                    .unwrap();
                let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) = admission else {
                    panic!("the selected caller must claim scheduling");
                };
                finish
                    .expect("selected scheduling prepares its real finish")
                    .finish(source, &mut result);
            }
        }
        result
            .err()
            .expect("scheduling keeps the full caller denial")
    }
}

#[test]
fn both_scheduling_finishes_preserve_claims_and_custody_on_interruption() {
    for finish in [Finish::Ordinary, Finish::Selected] {
        for kind in [
            WorthQueryOutputDemandDenialKind::Cancelled,
            WorthQueryOutputDemandDenialKind::TimedOut,
        ] {
            for reopened in [false, true] {
                let fixture = Fixture::new(reopened);
                let before = fixture.custody();
                assert!(before.0 > 0 && before.1 > 0 && before.2 > 0 && before.3 > 0);
                let denial = finish.stop(&fixture, kind.clone());
                assert_eq!(denial.kind(), kind);
                assert_eq!(
                    denial.recovery_posture(),
                    Posture::Retryable,
                    "{finish:?}: interruption leaves the row for a later claim"
                );
                assert_eq!(fixture.custody(), before, "no custody credit is released");
                {
                    let state = fixture.registry.state.lock().unwrap();
                    let row = &state.records[&fixture.interest.key];
                    assert_eq!(row.prerequisites.len(), 1);
                    assert_eq!(row.prerequisites[0].as_ref(), &fixture.upstream.key);
                    assert_eq!(row.framework_required_count, 1, "dependent claim retained");
                    assert_eq!(
                        state.records[&fixture.upstream.key].framework_required_count,
                        1
                    );
                    assert_eq!(state.records[&fixture.dependent.key].prerequisites.len(), 1);
                    assert_eq!(row.performed_obligations.len(), 1);
                    assert_eq!(row.performed_obligations[0].source_commit, fixture.commit);
                    assert_eq!(
                        row.performed_obligations[0].source,
                        fixture.interest.key.source
                    );
                    let source = row
                        .performed_source
                        .as_ref()
                        .expect("performed source retained");
                    assert_eq!(Arc::as_ptr(&source.change).cast::<()>(), fixture.change);
                    assert_eq!(
                        *source
                            .receipt
                            .committed_product_publication()
                            .composite_commit(),
                        fixture.commit
                    );
                    assert!(!row.pending_cleanup_queued);
                    if reopened {
                        let DemandState::Output(output) = &row.state else {
                            panic!("a reopened Ready is given back, not discarded");
                        };
                        assert!(matches!(
                            output.advancement,
                            WorthQueryOutputAdvancement::Idle
                        ));
                        let Some(WorthQueryOutputCheckpoint::Ready(ready)) = &output.checkpoint
                        else {
                            panic!("the exact accepted Ready remains");
                        };
                        assert_eq!(std::ptr::from_ref(&**ready).cast::<()>(), fixture.ready);
                        assert!(row.successor_of.is_none());
                    } else {
                        assert!(matches!(row.state, DemandState::Admitted));
                        assert_eq!(
                            super::super::succession::Succession::predecessor_of(&row.successor_of),
                            Some(*fixture.receipt.idempotency_binding().key_identity())
                        );
                    }
                }
                // Ready is certified and reopened before a later scheduling claim.
                let next = fixture.next_interest(reopened);
                assert!(matches!(
                    fixture.registry.begin(&next),
                    WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
                ));
            }
        }
    }
}

#[test]
fn other_scheduling_stops_still_fail_the_row_and_release_obligations() {
    for finish in [Finish::Ordinary, Finish::Selected] {
        let fixture = Fixture::new(true);
        let denial = finish.stop(
            &fixture,
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        );
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::ProducerUnavailable
        );
        assert_eq!(denial.recovery_posture(), Posture::Terminal);
        let state = fixture.registry.state.lock().unwrap();
        let row = &state.records[&fixture.interest.key];
        assert!(matches!(&row.state, DemandState::Failed(stored)
            if stored.kind() == WorthQueryOutputDemandDenialKind::ProducerUnavailable));
        assert!(row.performed_source.is_none());
        assert!(row.performed_obligations.is_empty());
        assert_eq!(state.obligation_reserved_bytes, 0);
    }
}

#[test]
fn interrupted_execution_reports_the_same_retryable_posture() {
    for selected in [false, true] {
        for kind in [
            WorthQueryOutputDemandDenialKind::Cancelled,
            WorthQueryOutputDemandDenialKind::TimedOut,
        ] {
            let fixture = Fixture::new(false);
            assert!(matches!(
                fixture.registry.begin(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
            ));
            fixture.registry.finish_scheduling(
                &fixture.interest,
                None,
                &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
            );
            let finish = selected.then(|| {
                fixture
                    .registry
                    .prepare_selected_execution_finish(&fixture.interest, &mut record_admission())
                    .unwrap()
            });
            assert!(matches!(
                fixture.registry.begin(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Execute { .. }
            ));
            let mut denial = WorthQueryOutputDemandDenial::new(kind, "execution stopped");
            if let Some(finish) = finish {
                finish.failure(&mut denial);
            } else {
                fixture
                    .registry
                    .finish_execution_failure(&fixture.interest, &mut denial);
            }
            assert_eq!(denial.recovery_posture(), Posture::Retryable);
        }
    }
}
