use super::support::*;
use crate::facade::RuntimeBridgeBuilder;

#[test]
fn build_freezes_mapping_registry_before_runtime_construction() {
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_registration("user-profile-name"))
        .register_aspect_mapping(exact_aspect_registration("user-profile-name-field"))
        .build()
        .expect("builder should freeze mapping registrations");

    assert_eq!(
        runtime.policy(),
        &crate::policy::BridgeRuntimePolicy::default()
    );
}
