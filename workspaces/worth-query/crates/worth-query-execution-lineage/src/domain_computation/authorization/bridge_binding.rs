pub fn bridge_authorization_binding_identity(
    identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
) -> worth_runtime_bridge::facade::BridgeAuthorizationBindingIdentity {
    worth_runtime_bridge::facade::BridgeAuthorizationBindingIdentity::new(
        identity.runtime_ordinal(),
        identity.generation(),
        *identity.package_identity(),
        *identity.schema_identity(),
    )
}
