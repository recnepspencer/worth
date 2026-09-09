use super::*;
use crate::conditional_execution::ContextRetentionCapture;
use crate::facade::{
    BridgeConditionalRetentionBudget, BridgeInstalledConditionalLowering,
    BridgeSealedRuntimeAssembly,
};

#[test]
fn provider_context_reserves_before_callbacks_and_retains_only_its_escaped_backing() {
    crate::conditional_execution::assert_context_retention(owner);
}

fn owner(
    budget: BridgeConditionalRetentionBudget,
    capture: Arc<ContextRetentionCapture>,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    let contacts = Arc::new(Contacts::default());
    let providers = CapturingProviders {
        capture,
        source: Providers(Arc::clone(&contacts)),
    };
    let (mut builder, request) = installation_fixture_with_runtime(
        super::super::super::semantic_dependencies::runtime_predicate_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new()
            .condition(providers.clone())
            .compute(providers),
        &[],
        |registrations| {
            let mapping = super::super::super::exact_mapping();
            let aspect = super::super::super::aspect_mapping(&mapping);
            let builder = RuntimeBridgeBuilder::new()
                .with_committed_patch_source(super::super::super::TestSource)
                .with_snapshot_read_source(PacketSource {
                    contacts: Arc::clone(&contacts),
                    wrong_packet: false,
                })
                .with_signal_sink(super::super::super::TestSink)
                .register_mapping(mapping)
                .register_aspect_mapping(aspect);
            let mut bridge = registrations
                .into_iter()
                .fold(builder, |builder, registration| {
                    builder.register_semantic_correspondence(registration)
                })
                .build()
                .unwrap();
            bridge.policy = bridge.policy.with_conditional_retention(budget);
            bridge
        },
    );
    let lowering = builder.install(request).unwrap();
    (builder.seal().unwrap(), lowering)
}

#[derive(Clone)]
struct CapturingProviders {
    capture: Arc<ContextRetentionCapture>,
    source: Providers,
}

impl BridgeConditionalProviderSemantics for CapturingProviders {
    type SemanticContract = ();
    fn semantic_contract(&self) {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        crate::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                crate::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.capture.as_ref(),
                ),
                crate::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.source.0.as_ref(),
                ),
            ],
            [],
        )
    }
}

impl BridgeConditionalConditionProvider for CapturingProviders {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        self.capture.predicates.fetch_add(1, Ordering::SeqCst);
        *self.capture.context.lock().unwrap() = Some(context.clone());
        self.source.resolve(context)
    }
}

impl BridgeConditionalComputeProvider for CapturingProviders {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.capture.computes.fetch_add(1, Ordering::SeqCst);
        self.source.compute(context)
    }
}
