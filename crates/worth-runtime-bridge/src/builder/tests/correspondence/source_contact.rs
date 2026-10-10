use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

struct CountingSource {
    envelope: BridgeCommittedPatchEnvelope,
    loads: Arc<AtomicUsize>,
}

impl crate::facade::CommittedPatchSource for CountingSource {
    fn authoritative_source_profile(
        &self,
    ) -> Option<crate::facade::BridgeAuthoritativeSourceProfile> {
        Some(
            crate::facade::BridgeAuthoritativeSourceProfile::new(99, "relational-adapter:99")
                .unwrap(),
        )
    }

    fn load_committed_patch(
        &self,
        _request: crate::facade::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, crate::facade::RelationalBridgeSourceError> {
        self.loads.fetch_add(1, Ordering::SeqCst);
        Ok(self.envelope.clone())
    }
}

#[test]
fn stale_bridge_runtime_denies_before_contacting_the_registered_source() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let registered = || registration(dependency("query:one"), vec![target(&graph, node)]);
    let owner_runtime = runtime_with_delivery_source(
        exact_mapping(),
        field_change_envelope_for_source_role("model"),
        vec![registered()],
    );
    let TransitionOutcome::Success(correspondence) =
        owner_runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("owner correspondence should install")
    };

    let loads = Arc::new(AtomicUsize::new(0));
    let mapping = exact_mapping();
    let foreign_runtime = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(CountingSource {
            envelope: field_change_envelope_for_source_role("model"),
            loads: Arc::clone(&loads),
        })
        .with_snapshot_read_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(mapping.clone())
        .register_aspect_mapping(super::semantic_fixture::aspect_mapping(&mapping))
        .register_semantic_correspondence(registered())
        .build()
        .unwrap();
    assert!(matches!(
        foreign_runtime.deliver_installed_correspondence(
            &correspondence,
            &mut graph,
            crate::facade::RelationalCommittedPatchRequest::new(truth_commit(1)),
            execution
        ),
        TransitionOutcome::Stale(_)
    ));
    assert_eq!(loads.load(Ordering::SeqCst), 0);
}
