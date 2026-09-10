use crate::runtime::tests::support::*;

pub(in crate::runtime::tests) struct TestProductRoot {
    pub(in crate::runtime::tests) source:
        worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    pub(in crate::runtime::tests) bridge: RuntimeBridge,
}

pub(in crate::runtime::tests) fn test_product_root() -> TestProductRoot {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-high-level-test-product")
        .build();
    test_product_root_with_runtime(runtime)
}

pub(in crate::runtime::tests) fn test_product_root_with_runtime(
    runtime: worth_relational::facade::runtime::RelationalRuntime,
) -> TestProductRoot {
    test_product_root_with_runtime_and_writeback(runtime, true)
}

pub(in crate::runtime::tests) fn test_product_root_without_writeback() -> TestProductRoot {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-high-level-test-product-without-writeback")
        .build();
    test_product_root_with_runtime_and_writeback(runtime, false)
}

fn test_product_root_with_runtime_and_writeback(
    runtime: worth_relational::facade::runtime::RelationalRuntime,
    install_writeback: bool,
) -> TestProductRoot {
    let source = worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
        runtime,
        "high-level-test-product",
    )
    .expect("the test Product source role should admit");
    let bridge = build_test_product_bridge(source.bridge_source(), install_writeback)
        .expect("the test Product bridge should derive from its Relational owner");
    TestProductRoot { source, bridge }
}

pub(in crate::runtime::tests) fn build_test_product_bridge(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
    install_writeback: bool,
) -> Result<RuntimeBridge, worth_runtime_bridge::facade::BridgeBuildError> {
    let aspect = AspectKey::new("aspect").expect("the test Product aspect should admit");
    let field = FieldKey::new("field").expect("the test Product field should admit");
    let bridge = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(TestBridgeSink)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("external-test"),
            TruthPatchScope::for_entity_field(MappingSelector::any(), aspect.clone(), field),
            SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("external-test"),
            CoarseRoutingMode::Direct,
        ));
    let bridge = if install_writeback {
        bridge.with_writeback_authority(StaticWritebackAuthority)
    } else {
        bridge
    }
    .build()?;
    Ok(bridge)
}
