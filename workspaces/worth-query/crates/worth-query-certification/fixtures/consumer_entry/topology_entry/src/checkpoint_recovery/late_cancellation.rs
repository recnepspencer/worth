//! A request stopped while waiting for the real final Product read cannot publish.

use super::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
    WorthQueryOutputSettlementPosture,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
};

#[test]
fn cancellation_during_final_product_guard_wait_refuses_stable_publication() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let performed = input_cutoff::direct_demand(&request, &application, "anchor-a");
    let original_receipt = performed.application_commit_receipt().unwrap().clone();
    let successor = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(successor.observed_sources()[0].clone())
        .idempotency(&301_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("a real source suffix changes before cutoff preparation");
    drop(successor);
    let before = request.retain_read().unwrap();
    let output_before = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let cancelled_scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    let cancellable_request = application.request(&principal, &cancelled_scope);
    let mut demand = cancellable_request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("the live request admits preparation");
    let pause = application.pause_product_currentness_for_test();
    std::thread::scope(|threads| {
        let advancing = threads.spawn(|| {
            for _ in 0..256 {
                match demand.advance(&cancellable_request) {
                    Ok(WorthQueryApplicationOutputDemandProgress::Pending) => {
                        std::thread::yield_now()
                    }
                    terminal => return terminal,
                }
            }
            panic!("the bounded progression never reached its final guard");
        });
        let reached = pause.wait_until_reader_contended(Duration::from_secs(5));
        cancellation.cancel();
        pause.release();
        let terminal = advancing.join().unwrap();
        assert!(
            reached,
            "the real final Product read must contend after preparation"
        );
        assert!(
            matches!(terminal,
                Err(WorthQueryApplicationOutputDemandDenial::Demand(ref denial))
                    if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled
            ),
            "late cancellation must be the typed refusal"
        );
    });
    drop(pause);
    drop(demand);
    let after = request.retain_read().unwrap();
    assert_eq!(before.selected_commit(), after.selected_commit());
    let output_after = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(output_before.rows(), output_after.rows());
    assert_eq!(application.world_active_publication_attempts_for_test(), 0);
    assert_eq!(application.active_application_attempts_for_test(), 0);
    drop((before, after, output_before, output_after));

    let retried = input_cutoff::direct_demand(&request, &application, "anchor-a");
    assert_eq!(
        retried.posture(),
        WorthQueryOutputSettlementPosture::StableReused
    );
    assert_eq!(retried.producer_contacts_in_this_demand(), 0);
    assert!(retried.application_commit_receipt().is_none());
    let original = original_receipt
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            PlanarMutationBinding<CheckpointSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::from_static("anchor"))
        .unwrap()
        .entity_id();
    let current = retried
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            PlanarMutationBinding<CheckpointSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::from_static("anchor"))
        .unwrap()
        .entity_id();
    assert_eq!(original, current);
}
