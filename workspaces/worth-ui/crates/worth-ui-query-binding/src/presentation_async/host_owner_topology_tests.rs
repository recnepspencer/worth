use super::tests::{basis, installed_owner};
use super::*;

#[test]
fn presentation_host_installs_only_its_async_domain() {
    let plan = WorthUiPresentationAsyncHostPlan::prepare().unwrap();
    let (request, _) = plan.into_parts();
    let packages = request.into_packages();
    let [package] = packages.as_slice() else {
        panic!("presentation custody must install exactly one Query domain");
    };
    assert_eq!(
        package.package().domain_identity().owner(),
        "WORTH.ui.presentation-async"
    );
}

#[test]
fn host_plan_retains_one_query_owned_graph_and_pending_attempts() {
    let mut owner = installed_owner();
    let baseline = basis(1);
    let graph = owner
        .workspace
        .owned_async_runtime_topology()
        .unwrap()
        .signal_graph_instance();
    let receipt = owner.admit_pending(baseline.clone()).unwrap();

    assert_eq!(
        receipt.observation().posture(),
        WorthUiPresentationAsyncPosture::Pending
    );
    assert_eq!(receipt.observation().signal_graph_instance(), graph);
    assert!(matches!(
        owner.admit_pending(baseline),
        Err(WorthUiPresentationPendingAdmissionDenial::DuplicateAttemptBinding)
    ));
    let successor = owner.admit_pending(basis(2)).unwrap();
    assert_eq!(successor.observation().signal_graph_instance(), graph);
    assert_eq!(owner.pending.len(), 2);
}

#[test]
fn independent_query_hosts_cannot_share_semantic_signal_identity_or_handles() {
    let mut left = installed_owner();
    let mut right = installed_owner();
    let left_graph = left
        .workspace
        .owned_async_runtime_topology()
        .unwrap()
        .signal_graph_instance();
    let right_graph = right
        .workspace
        .owned_async_runtime_topology()
        .unwrap()
        .signal_graph_instance();
    assert_ne!(left_graph, right_graph);

    let left_receipt = left.admit_pending(basis(11)).unwrap();
    let right_receipt = right.admit_pending(basis(12)).unwrap();
    assert_eq!(
        left_receipt.observation().signal_graph_instance(),
        left_graph
    );
    assert_eq!(
        right_receipt.observation().signal_graph_instance(),
        right_graph
    );
    assert_ne!(
        left_receipt.observation().signal_graph_instance(),
        right_receipt.observation().signal_graph_instance()
    );
}
