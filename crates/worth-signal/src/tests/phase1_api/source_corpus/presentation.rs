pub(in crate::tests::phase1_api) const DOT_SOURCE: &str =
    include_str!("../../../presentation/outputs/dot.rs");
pub(in crate::tests::phase1_api) const HARNESS_BRIDGE_SOURCE: &str = concat!(
    include_str!("../../../presentation/harness/bridge.rs"),
    include_str!("../../../presentation/harness/bridge/adapter.rs"),
    include_str!("../../../presentation/harness/bridge/projection.rs"),
    include_str!("../../../presentation/harness/bridge/request.rs"),
    include_str!("../../../presentation/harness/bridge/translation.rs"),
);
