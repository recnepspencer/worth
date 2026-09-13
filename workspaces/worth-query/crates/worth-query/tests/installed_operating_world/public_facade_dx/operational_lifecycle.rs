use super::*;

#[test]
fn bound_operations_drive_native_live_invalidation_collection_and_disposal() {
    let mut workspace = workspace("installed-public-facade-single-root", false).unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let root = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let reads = root.family(ReadFamily);
    let subject = reads.bind(&domain, ReadVertex).unwrap();
    let compatibility_peer = reads.bind(&domain, ReadVertex).unwrap();

    subject.same_installation_with(&compatibility_peer).unwrap();
    subject.compatible_basis_with(&compatibility_peer).unwrap();
    let snapshot_contract = subject.consumer_projection_contract().unwrap();
    let mut snapshot_request = snapshot_contract.projection_request();
    let native_selection = snapshot_request
        .select_display_native_field_name("id")
        .unwrap();
    let snapshot_request = snapshot_request.build().unwrap();
    let native_key = snapshot_request
        .resolve_native_key(&native_selection)
        .unwrap()
        .into_key();
    let snapshot = subject
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(snapshot_request)
        .unwrap()
        .settle()
        .unwrap();
    assert!(snapshot.native_value(&native_key, 0).is_ok());

    let mut collection_workspace = matrix_workspace("installed-public-facade-collection", 3, false);
    let collection_domain = collection_workspace.domain(GeometryDomain).unwrap();
    let collection_root = collection_workspace
        .observe_operating_world(collection_workspace.current_world())
        .unwrap();
    let collection_family = collection_root.family(ReadFamily);
    let collection_bound = collection_family
        .bind(&collection_domain, NativeMatrixRead)
        .unwrap();
    let live_bound = collection_family
        .bind(&collection_domain, NativeMatrixRead)
        .unwrap();
    let collection_contract = collection_bound.consumer_projection_contract().unwrap();
    let mut collection_request = collection_contract.projection_request();
    collection_request
        .select_derived_native_field_name("f15")
        .unwrap();
    let collection_projection = collection_bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &collection_workspace,
        )
        .unwrap()
        .execute(&mut collection_workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(collection_request.build().unwrap())
        .unwrap()
        .settle()
        .unwrap();
    let collection = collection_projection.into_bound_collection().unwrap();
    let breadth =
        installed::collection::WorthQueryCollectionWindowBreadth::new(8, 1, 1, 10).unwrap();
    let admitted = collection
        .declare_window(collection.beginning_cursor(), breadth)
        .unwrap();
    let window = collection.resolve_window(admitted).unwrap();
    let mut consumer =
        installed::collection::WorthQueryCollectionConsumerWindow::from_bound(collection, window)
            .unwrap();

    let live_contract = live_bound.consumer_projection_contract().unwrap();
    let mut live_request = live_contract.projection_request();
    live_request
        .select_derived_native_field_name("f15")
        .unwrap();
    let live_projection = live_bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &collection_workspace,
        )
        .unwrap()
        .execute(&mut collection_workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(live_request.build().unwrap())
        .unwrap()
        .settle()
        .unwrap();
    let promoted = match live_projection
        .into_lifecycle()
        .promote(&mut collection_workspace)
    {
        installed::observation::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("ordinary facade did not promote the current projection"),
    };
    let lease = match promoted.into_managed_lease(&mut collection_workspace) {
        installed::observation::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
        installed::observation::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
            panic!(
                "ordinary facade did not admit a managed lease: {}",
                stop.detail()
            )
        }
    };

    assert!(lease
        .drain(&mut collection_workspace)
        .unwrap()
        .delivery()
        .is_empty());
    insert_matrix_value(
        &mut collection_workspace,
        3,
        matrix_value_with_order(3, "public-facade-invalidation"),
    );
    let delivery = lease.drain(&mut collection_workspace).unwrap();
    let delta = lease.consumer_invalidation_delta(delivery).unwrap();
    let admitted = match lease.admit_consumer_invalidation_delta(delta, &collection_workspace) {
        Ok(admitted) => admitted,
        Err(stop) => panic!("current invalidation did not readmit: {:?}", stop.kind()),
    };
    consumer
        .bind_shared_target(&admitted, &collection_workspace)
        .unwrap();
    let patch = match consumer.plan_patch(&admitted, &collection_workspace) {
        installed::collection::WorthQueryCollectionDeliveryOutcome::Patch(patch) => patch,
        installed::collection::WorthQueryCollectionDeliveryOutcome::NoDelivery(denial) => {
            panic!("semantic invalidation produced no collection patch: {denial:?}")
        }
    };
    let receipt = consumer.apply_patch(patch).unwrap();
    assert!(!receipt.operations().is_empty());

    let disposed = match lease.dispose(&mut collection_workspace) {
        installed::observation::WorthQuerySharedProjectionDisposalOutcome::Disposed(receipt) => {
            receipt
        }
        installed::observation::WorthQuerySharedProjectionDisposalOutcome::Stopped(stop) => {
            panic!("ordinary facade disposal stopped: {}", stop.error())
        }
    };
    assert!(disposed.release().owner_terminal());
    assert_eq!(disposed.release().counters().close_completions, 1);
}
