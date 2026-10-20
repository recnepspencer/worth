//! Real assembly identities and an early route fence; no Query permission or persistence claim.
use crate::{
    surfaces::CompatHttpSurface, WorthServer, WorthServerConfig, WorthServerOperationRegistration,
    WorthServerRouteBranchTarget, WorthServerRouteTransportRequest, WorthServerTransportDenialCode,
};

fn server() -> WorthServer {
    WorthServer::builder()
        .with_instance_selection_policy(
            crate::WorthServerInstanceSelectionPolicy::RequiredForProductCalls,
        )
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 0).into())
                .with_middleware_config(
                    crate::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .register_operations(WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .unwrap()
}

#[test]
fn assemblies_share_their_own_identity_and_refuse_old_selection_before_body_admission() {
    assert!(
        crate::WorthServerInstanceSelectionPolicy::RequiredForProductCalls.requires_selection(
            crate::WorthServerOperationFamily::ProductApplicationRead,
            "product_session.open_mutation",
        ),
        "a business call cannot obtain the discovery exemption from its name alone"
    );
    let original = server();
    let replacement = server();
    assert_ne!(
        original.instance_identity(),
        replacement.instance_identity()
    );
    assert_eq!(
        original.product_protocol_catalog().unwrap(),
        replacement.product_protocol_catalog().unwrap(),
        "live nonces must not enter the static catalog"
    );
    let catalog = replacement.product_protocol_catalog().unwrap();
    for operation in catalog.product_session_operations() {
        assert_eq!(
            operation.requires_server_instance(),
            operation.operation_name() == "product_session.close"
        );
    }
    assert_eq!(
        replacement.instance_identity(),
        replacement.product_session_registry().instance_identity()
    );
    let bridge = replacement
        .operation_router()
        .bridge_for("POST", "/compat/mutations/product_session.open_mutation")
        .unwrap();
    let request = |instance: &str| {
        WorthServerRouteTransportRequest::new(
            "principal",
            "tenant",
            "workspace",
            WorthServerRouteBranchTarget::Main,
        )
        .with_header("x-worth-server-instance", instance)
        .with_raw_body(b"not JSON".to_vec(), Some("application/json"))
    };
    let refused = bridge
        .execute(request(original.instance_identity().as_str()))
        .unwrap_err();
    assert_eq!(
        refused.code(),
        WorthServerTransportDenialCode::ServerInstanceMismatch
    );
    assert_eq!(
        refused.requested_server_instance(),
        Some(original.instance_identity().as_str())
    );
    assert_eq!(
        refused.actual_server_instance(),
        Some(replacement.instance_identity().as_str())
    );
    let admitted = bridge
        .execute(request(replacement.instance_identity().as_str()))
        .unwrap_err();
    assert_eq!(
        admitted.code(),
        WorthServerTransportDenialCode::MalformedJson,
        "{admitted:?}"
    );
    let close = replacement
        .operation_router()
        .bridge_for("POST", "/compat/mutations/product_session.close")
        .unwrap();
    let missing = WorthServerRouteTransportRequest::new(
        "principal",
        "tenant",
        "workspace",
        WorthServerRouteBranchTarget::Main,
    )
    .with_raw_body(b"not JSON".to_vec(), Some("application/json"));
    assert_eq!(
        close.execute(missing.clone()).unwrap_err().code(),
        WorthServerTransportDenialCode::MissingServerInstance
    );
    // Discovery alone may be unselected, and then reaches ordinary body admission.
    assert_eq!(
        bridge.execute(missing).unwrap_err().code(),
        WorthServerTransportDenialCode::MalformedJson
    );
}
