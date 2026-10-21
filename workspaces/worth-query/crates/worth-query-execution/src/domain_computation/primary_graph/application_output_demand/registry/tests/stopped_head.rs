//! Every stopped-checkpoint finish releases its exact head hold.
use super::super::{
    WorthQueryOutputAdvancement, WorthQueryOutputDemandAdvanceAdmission as Admission,
};
use super::published_retirement::{admit, publish};
use super::*;

#[test]
fn stopped_head_is_released_by_every_finish_route() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        for route in 0..3 {
            let registry = WorthQueryOutputDemandRegistry::default();
            let row = key("stopped-head", 1, 1);
            let owner = admit(&registry, &row);
            let peer = admit(&registry, &row);
            publish(&registry, &owner, receipt.clone());
            assert_eq!(
                registry
                    .state
                    .lock()
                    .unwrap()
                    .records
                    .publication_count_for_test(),
                1
            );
            let stop = WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ProductDelivery(Box::new(
                    crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind::ForeignProductRoot)), "");
            match route {
                0 => {
                    let mut state = registry.state.lock().unwrap();
                    let DemandState::Output(output) =
                        &mut state.records.get_mut(&row).unwrap().state
                    else {
                        panic!("published row")
                    };
                    output.stop(stop.clone());
                }
                1 => {
                    let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin(&owner)
                    else {
                        panic!("Published is claimed")
                    };
                    registry
                        .finish_checkpoint(&owner, claim, checkpoint, Some(&stop))
                        .unwrap();
                }
                2 => {
                    let (finish, claim, checkpoint) = registry
                        .prepare_selected_checkpoint_finish(&owner)
                        .unwrap()
                        .claim()
                        .unwrap()
                        .unwrap();
                    assert!(finish.finish(claim, checkpoint, Some(&stop)).is_ok());
                }
                _ => unreachable!(),
            }
            let state = registry.state.lock().unwrap();
            let count = state.records.publication_count_for_test();
            let DemandState::Output(output) = &state.records[&row].state else {
                panic!("checkpoint retained")
            };
            assert!(matches!(
                output.advancement(),
                WorthQueryOutputAdvancement::Stopped { .. }
            ));
            assert_eq!(
                count, 0,
                "route {route}: a stopped row cannot freeze its head"
            );
            drop(state);
            assert!(
                matches!(registry.begin(&peer), Admission::Failed(ref denial) if denial.kind() == stop.kind())
            );
        }
    });
}
