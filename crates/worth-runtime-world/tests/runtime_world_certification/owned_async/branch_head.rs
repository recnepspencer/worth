use super::fixture::AsyncSourceCourt;
use worth_runtime_bridge::facade::BridgeAsyncRequestTruthViewBasisKind;
use worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial as Denial;

#[test]
fn observed_empty_head_preserves_snapshot_without_claiming_a_commit() {
    let court = AsyncSourceCourt::new();
    let source = court.lease(&court.empty);
    assert!(court
        .empty
        .basis()
        .relational_basis()
        .observation()
        .commit_id()
        .is_none());
    assert!(matches!(
        court.world.admit_unqueued_owned_async_request(
            &court.bridge,
            &court.declaration,
            &court.empty,
            &source,
        ),
        Err(Denial::NoCommittedSource)
    ));

    let foreign = AsyncSourceCourt::new();
    let foreign_source = foreign.lease(&foreign.empty);
    assert!(matches!(
        court
            .world
            .admit_unqueued_owned_async_request_at_branch_head(
                &court.bridge,
                &court.declaration,
                &foreign.empty,
                &foreign_source,
            ),
        Err(Denial::ForeignOwner)
    ));
    assert!(matches!(
        court
            .world
            .admit_unqueued_owned_async_request_at_branch_head(
                &court.bridge,
                &court.declaration,
                &court.empty,
                &foreign_source,
            ),
        Err(Denial::RelationalSourceMismatch)
    ));

    let mut owner = court
        .world
        .admit_unqueued_owned_async_request_at_branch_head(
            &court.bridge,
            &court.declaration,
            &court.empty,
            &source,
        )
        .unwrap();
    assert!(owner.refusal().is_none());
    let witness = owner.retained_request().unwrap();
    let truth = witness.request().basis_binding().truth_view_basis();
    assert_eq!(
        truth.kind(),
        BridgeAsyncRequestTruthViewBasisKind::BranchHead
    );
    assert!(truth.truth_commit_identity().is_none());
    assert_eq!(
        truth.truth_snapshot_identity(),
        Some(source.snapshot_identity())
    );
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&witness)
            .unwrap(),
        1
    );

    // A foreign source refusal cannot supersede the genuine active occurrence.
    assert!(matches!(
        court
            .world
            .admit_unqueued_owned_async_request_at_branch_head(
                &court.bridge,
                &court.declaration,
                &court.empty,
                &foreign_source,
            ),
        Err(Denial::RelationalSourceMismatch)
    ));
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&witness)
            .unwrap(),
        1
    );
    court
        .bridge
        .finish_unqueued_owned_async_entry_cleanup(&mut owner)
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&witness)
            .unwrap(),
        0
    );
}
