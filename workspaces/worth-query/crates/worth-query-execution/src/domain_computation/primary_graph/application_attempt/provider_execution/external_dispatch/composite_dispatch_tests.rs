use std::sync::atomic::{AtomicUsize, Ordering};

use crate::domain_computation::application_aftermath::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};
use crate::domain_computation::primary_graph::two_recoverable_application_commits;

use super::WorthQueryExternalDispatchPreparationDenial;

struct CountingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CountingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn sibling_publication_substitution_denies_before_transport() {
    let (world, first, second) = two_recoverable_application_commits(185, 186);
    let transport = CountingTransport(AtomicUsize::new(0));
    let substituted = world
        .application
        .observe_committed_dispatch_outbox(&first)
        .unwrap()
        .unwrap()
        .with_product_publication_for_test(second.committed_product_publication().clone());
    assert_eq!(
        substituted.relational_runtime_instance_id(),
        first.provider_runtime_instance_id()
    );
    assert_eq!(
        substituted.committed_product_publication().product_branch(),
        first.committed_product_publication().product_branch()
    );
    assert_ne!(
        substituted.commit_reference(),
        substituted
            .committed_product_publication()
            .relational_commit()
    );

    assert_eq!(
        world
            .application
            .perform_committed_external_dispatch(&transport, substituted),
        Err(WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied)
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 0);
}
