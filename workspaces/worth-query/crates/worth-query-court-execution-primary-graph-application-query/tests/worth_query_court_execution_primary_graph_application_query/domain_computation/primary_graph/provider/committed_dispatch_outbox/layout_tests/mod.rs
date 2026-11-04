//! Independent physical-name oracle for the installed outbox projection.
//!
//! The production lowerer and the persistence/restoration code share a typed
//! layout. These assertions deliberately derive the expected locators from
//! frozen physical names instead, so swapping two same-typed layout members
//! cannot make both sides agree and silently certify the wrong schema.

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn installed_outbox_layout_preserves_every_frozen_physical_field_name() {
    let world = installed_authorization_world(true);
    let layout = (*(*(*world
        .application
        .primary_provider_for_test())
        .graph_for_test())
        .layout_for_test())
        .provider_dispatch_outbox_for_test();
    let expected = |field: &str| {
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("dispatch-outbox").expect("frozen aspect key"),
            CanonicalFieldPath::single(FieldKey::new(field).expect("frozen field key")),
        )
    };

    assert_eq!(layout.correlation_locator, expected("correlation"));
    assert_eq!(layout.family_locator, expected("correlation-family"));
    assert_eq!(layout.effect_locator, expected("effect"));
    assert_eq!(
        layout.protocol_identity_locator,
        expected("protocol-identity")
    );
    assert_eq!(
        layout.protocol_version_locator,
        expected("protocol-version")
    );
    assert_eq!(
        layout.maximum_payload_bytes_locator,
        expected("maximum-payload-bytes")
    );
    assert_eq!(layout.payload_locator, expected("payload-hex"));
    assert_eq!(
        layout.outcome_identity_locator,
        expected("outcome-identity")
    );
    assert_eq!(layout.operation_slot_locator, expected("operation-slot"));
}
