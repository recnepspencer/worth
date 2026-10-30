pub(crate) use worth_query_testkit_query::facade::query::suite::operation_invalidation::*;

#[test]
fn consequence_authority_requires_the_exact_current_lease_and_stays_consumer_authored() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, subject, candidate) =
        shared_native_leases(request_execution, "invalidation-readmission");
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "readmit-update")
        })
        .unwrap();
    let delivery = subject.drain(&mut workspace).unwrap();
    let _peer = candidate.drain(&mut workspace).unwrap();
    let delta = subject.consumer_invalidation_delta(delivery).unwrap();

    let stop = match candidate.admit_consumer_invalidation_delta(delta, &workspace) {
        Err(stop) => stop,
        Ok(_) => panic!("foreign lease admitted the subject delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease
    );
    let delta = stop.into_delta();
    let admitted = match subject.admit_consumer_invalidation_delta(delta, &workspace) {
        Ok(admitted) => admitted,
        Err(_) => panic!("exact current lease did not readmit its delta"),
    };
    #[derive(Debug, Eq, PartialEq)]
    enum UiAction {
        PatchMountedField,
    }
    #[derive(Debug, Eq, PartialEq)]
    enum CacheAction {
        EvictEntry,
    }
    let first = match admitted.attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch,
        UiAction::PatchMountedField,
    ) {
        Ok(consequence) => consequence,
        Err(_) => panic!("exact UI patch consequence was denied"),
    };
    let second = match admitted.attach_consumer_authored_consequence(
        &workspace,
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute,
        CacheAction::EvictEntry,
    ) {
        Ok(consequence) => consequence,
        Err(_) => panic!("explicitly widened cache consequence was denied"),
    };
    assert_eq!(first.consumer_authored(), &UiAction::PatchMountedField);
    assert_eq!(second.consumer_authored(), &CacheAction::EvictEntry);
    assert_eq!(
        first.required_disposition(),
        domain::WorthQueryConsumerInvalidationDisposition::LocalPatch
    );
    assert_eq!(
        second.admitted_disposition(),
        domain::WorthQueryConsumerInvalidationDisposition::Reexecute
    );
    assert!(first
        .authority()
        .is_same_current_authority_as(admitted.delta().authority()));
    assert!(second
        .authority()
        .is_same_current_authority_as(admitted.delta().authority()));
}

#[test]
fn empty_epoch_cannot_mint_an_invalidation_delta() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, subject, candidate) =
        shared_native_leases(request_execution, "invalidation-empty");
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let _candidate_delivery = candidate.drain(&mut workspace).unwrap();
    let stop = match subject.consumer_invalidation_delta(subject_delivery) {
        Err(stop) => stop,
        Ok(_) => panic!("empty delivery minted an invalidation delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::NoSemanticDelivery
    );
    assert_eq!(stop.counters().lease_impact_readmission_attempts, 1);
    assert_eq!(stop.counters().semantic_delivery_checks, 1);
    assert_eq!(stop.counters().targeted_lease_deliveries, 0);
}

#[test]
fn unsupported_invalidation_posture_stops_before_delta_translation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, subject, candidate) = shared_native_leases_with_invalidation(
        request_execution,
        "invalidation-support-stop",
        domain::WorthQueryConsumerSupportPosture::Deferred,
    );
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    workspace
        .insert(request_execution, "Vertex", |mutation| {
            mutation.aspect("identity.id", "unsupported-invalidation")
        })
        .unwrap();
    let delivery = subject.drain(&mut workspace).unwrap();
    let _peer = candidate.drain(&mut workspace).unwrap();
    let stop = match subject.consumer_invalidation_delta(delivery) {
        Err(stop) => stop,
        Ok(_) => panic!("deferred invalidation support minted a delta"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryConsumerInvalidationDeltaStopKind::ConsumerSupportUnavailable
    );
    assert_eq!(
        stop.support_dimension(),
        Some(domain::WorthQueryConsumerSupportDimension::Invalidation)
    );
    assert_eq!(
        stop.support_posture(),
        Some(domain::WorthQueryConsumerSupportPosture::Deferred)
    );
    assert_eq!(stop.counters().consumer_support_checks, 1);
    assert_eq!(stop.counters().disposition_classifications, 0);
    assert_eq!(stop.counters().native_access_layout_lookups, 0);
}
