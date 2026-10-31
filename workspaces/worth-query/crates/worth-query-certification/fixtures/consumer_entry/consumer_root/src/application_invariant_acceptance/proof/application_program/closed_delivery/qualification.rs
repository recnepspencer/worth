//! Read-origin provenance and complete-tree qualification at an explicit basis.
use super::*;
use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCleanupDenial, WorthQueryApplicationProductBranchCloseDenial,
    WorthQueryProductBranchOwnerCleanupDenial,
};

#[test]
fn read_origin_rebuild_and_receiptless_reuse_qualify_the_original_complete_tree() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let (performed_fact, reused_fact) = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world
            .application
            .request(&principal, &scope)
            .on_branch(branch);
        edit_source(&world, &request, "anchor-a", 2, 0x9182_3030);
        let opening = request.retain_current_read().unwrap();
        let tree = settle_tree(&world, &request);
        let settlement = tree.settlement();
        assert!(settlement.performed_source().is_none());
        assert_eq!(
            settlement.root_settlement().posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        assert_eq!(settlement.output_count(), 4);
        assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
        let root = settlement.root_observation().selected_commit().clone();
        let root_basis = settlement.root_observation().retain();
        let common = request.retain_current_read().unwrap();
        assert_ne!(
            opening.selected_commit(),
            &root,
            "genuine producer publication follows read admission"
        );
        assert_ne!(
            &root,
            common.selected_commit(),
            "dependent publications follow the root"
        );
        let closed = tree
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("original native close: {:?}", stop.denial()));
        let root_only = request.at(&root_basis);
        root_only
            .require_current_output_demand(closed.settlement().root_settlement())
            .expect("the root is genuinely ready at its own original publication");
        let incomplete = closed
            .qualify_at(&root_only)
            .map(|_| ())
            .expect_err("a basis before dependent publications cannot qualify the complete tree");
        assert!(
            matches!(
                incomplete,
                WorthQueryOperationDeliveryDenial::Currentness(
                    WorthQueryOutputCurrentnessDenial::Output(_)
                        | WorthQueryOutputCurrentnessDenial::Source(_)
                )
            ),
            "the original complete set itself must be noncurrent: {incomplete:?}"
        );
        let performed = closed.qualify_at(&request.at(&common)).unwrap();
        assert_eq!(performed.source_commit(), opening.selected_commit());
        assert_eq!(performed.root_output_commit(), &root);
        assert_eq!(performed.output_commit(), common.selected_commit());
        assert_eq!(
            performed.root_posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        assert_eq!(
            request
                .at(&common)
                .query(PlanarOutputRead {
                    body_key: "anchor-a".into()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(3)
        );

        // A genuine independent source change forces root readmission instead
        // of simply rejoining the original performed receipt.
        change_suffix(&world, &request);
        let reuse_source = request.retain_current_read().unwrap();
        let reused = settle_tree(&world, &request);
        assert!(reused.settlement().performed_source().is_none());
        assert!(reused.settlement().root_receipt().is_none());
        assert_eq!(
            reused.settlement().root_settlement().posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(
            reused.settlement().root_producer_contacts_in_this_demand(),
            0
        );
        let closed_reused = reused
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("reused native close: {:?}", stop.denial()));
        let reuse_output = request.retain_current_read().unwrap();
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let reused_fact = closed_reused
            .qualify_at(&request.at(&reuse_output))
            .unwrap();
        assert_eq!(reused_fact.source_commit(), reuse_source.selected_commit());
        assert_eq!(reused_fact.output_commit(), reuse_output.selected_commit());
        assert_eq!(
            reused_fact.root_posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );

        // Native retained currentness sees this actual consumed-source movement.
        edit_source(&world, &request, "anchor-b", 4, 0x9182_3031);
        let later = request.retain_current_read().unwrap();
        let pinned = request.at(&later);
        assert!(
            matches!(closed_reused.qualify_at(&pinned),
                Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause)))
                    if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
            ),
            "the original tree cannot qualify after consumed-source supersession"
        );
        let historical = closed_reused
            .qualify_at(&request.at(&reuse_output))
            .expect("explicit historical E remains valid despite later current-head movement");
        assert_eq!(historical.output_commit(), reuse_output.selected_commit());
        assert_ne!(historical.output_commit(), later.selected_commit());
        assert_eq!(
            performed.output_commit(),
            common.selected_commit(),
            "issued fact remains retrospective"
        );
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "qualification and refusal never reconstruct or contact producers"
        );
        drop((closed, closed_reused, historical));
        (performed, reused_fact)
    };
    let pending = world
        .application
        .with_host_advancement(|phase| world.application.on_branch(branch).close(&phase))
        .unwrap();
    let pending = match pending {
        Err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(pending)) => pending,
        other => panic!("qualified facts retain exact child history after carrier drop: {other:?}"),
    };
    assert!(matches!(
        pending.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(performed_fact);
    let pending = pending
        .into_cleanup()
        .retry()
        .expect_err("the second qualified holder independently pins history");
    assert!(matches!(
        pending.denial(),
        WorthQueryApplicationProductBranchCleanupDenial::Product(
            WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
        )
    ));
    drop(reused_fact);
    assert!(pending
        .into_cleanup()
        .retry()
        .expect("actual physical history closes after the final qualified pin")
        .is_complete());
    world
        .application
        .close_conditional_runtime()
        .expect("all native custody closed and both qualified source/output leases released");
}
