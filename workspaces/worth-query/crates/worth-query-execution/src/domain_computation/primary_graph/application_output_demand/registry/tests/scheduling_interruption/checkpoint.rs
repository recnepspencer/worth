//! The same caller stop classification applies after publication and delivery.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Advance, WorthQueryManagedComputationResourceDenial as Resource,
};

#[test]
fn checkpoint_finishes_keep_caller_stops_off_the_shared_row() {
    use WorthQueryOutputDemandDenialKind as Kind;
    for selected in [false, true] {
        for (kind, fails_row) in [
            (Kind::Cancelled, false),
            (Kind::TimedOut, false),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::WorkExhausted)),
                false,
            ),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::RetainedBytesExhausted)),
                false,
            ),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::ScratchCapacityExceeded)),
                false,
            ),
            (Kind::ProducerUnavailable, true),
        ] {
            let fixture = Fixture::new(false);
            {
                let mut state = fixture.registry.state.lock().unwrap();
                state.records.get_mut(&fixture.interest.key).unwrap().state = DemandState::Output(
                    WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Published {
                        receipt: fixture.receipt.clone(),
                        delivery: super::super::super::WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: PreparedReadyBacking::for_test(),
                    }),
                );
            }
            let before = fixture.custody();
            let prepared = selected.then(|| {
                fixture
                    .registry
                    .prepare_selected_checkpoint_finish(&fixture.interest, &mut record_admission())
                    .unwrap()
            });
            let (prepared, claim, checkpoint) = if let Some(prepared) = prepared {
                let (prepared, claim, checkpoint) = prepared
                    .claim()
                    .unwrap()
                    .expect("published row is claimable");
                (Some(prepared), claim, checkpoint)
            } else {
                let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
                    fixture.registry.begin(&fixture.interest)
                else {
                    panic!("published row is claimable");
                };
                (None, claim, checkpoint)
            };
            let denial = WorthQueryOutputDemandDenial::new(kind, "caller stopped delivery");
            if let Some(prepared) = prepared {
                assert!(prepared.finish(claim, checkpoint, Some(&denial)).is_ok());
            } else {
                fixture
                    .registry
                    .finish_checkpoint(&fixture.interest, claim, checkpoint, Some(&denial))
                    .unwrap();
            }
            {
                let state = fixture.registry.state.lock().unwrap();
                let DemandState::Output(output) = &state.records[&fixture.interest.key].state
                else {
                    panic!("publication remains retained");
                };
                assert!(output.checkpoint.is_some());
                if fails_row {
                    assert!(
                        matches!(&output.advancement, WorthQueryOutputAdvancement::Stopped { denial, .. }
                        if denial.kind() == kind)
                    );
                } else {
                    assert!(matches!(
                        output.advancement,
                        WorthQueryOutputAdvancement::Idle
                    ));
                }
            }
            if !fails_row {
                assert_eq!(fixture.custody(), before);
                assert!(matches!(
                    fixture.registry.begin(&fixture.interest),
                    WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { .. }
                ));
            } else {
                assert!(matches!(fixture.registry.begin(&fixture.interest),
                    WorthQueryOutputDemandAdvanceAdmission::Failed(denial) if denial.kind() == kind));
            }
        }
    }
}
