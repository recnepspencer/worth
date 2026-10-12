use super::*;

#[test]
fn detached_bridge_compute_provider_is_rejected_instead_of_replaced() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let node = node(
        "detached-compute",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut installation = conditional_installation(&node);
    let detached_contacts = Arc::new(AtomicUsize::new(0));
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .compute(DetachedCompute(Arc::clone(&detached_contacts)));
    let query_contacts = Arc::new(AtomicUsize::new(0));

    let result = conditional_workspace_with(
        request_execution,
        "detached-compute",
        node,
        installation,
        CountedCompute(Arc::clone(&query_contacts)),
    );

    let Err(error) = result else {
        panic!("a detached Bridge compute provider must reject runtime construction")
    };
    assert!(error.message().contains("ExtraComputeProvider"));
    assert_eq!(detached_contacts.load(Ordering::SeqCst), 0);
    assert_eq!(query_contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn two_declarations_cannot_implicitly_share_one_signal_node() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = node(
        "first-owner",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let second = node(
        "second-owner",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );

    let result =
        shared_signal_node_workspace(request_execution, "shared-signal-node", second, first);

    let Err(error) = result else {
        panic!("implicit Signal-node sharing must reject runtime construction")
    };
    assert!(error.message().contains("SignalNodeAlreadyBound"));
}
