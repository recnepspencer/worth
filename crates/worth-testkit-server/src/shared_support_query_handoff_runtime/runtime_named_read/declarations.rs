pub use {worth_query::facade::foundation::AspectFieldKey};
pub use {worth_query::facade::runtime::WorthQueryWorkspace};
pub use {worth_server::WorthServerDirectDeclarationSourceKind, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryWorkspaceBindingError, worth_server::WorthServerQueryWorkspaceBindingRequest, worth_server::WorthServerQueryWorkspaceBindingTarget};

pub fn install_requested_named_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let Some(binding_label) = binding_label(request.target()) else {
        return Ok(());
    };

    if binding_label.ends_with(".missing") {
        return Ok(());
    }

    workspace
        .live_view::<serde_json::Value>(request_execution, binding_label, |q| {
            q.from("User")
                .select([
                    aspect_field_key("identity", "id"),
                    aspect_field_key("profile", "display_name"),
                ])
                .schema_basis("worth-server-test-named-read")
        })
        .map(|_| ())
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new(
                "workspace_declaration",
                format!("{error:?}"),
            )
        })
}

pub fn aspect_field_key(aspect: &str, field: &str) -> AspectFieldKey {
    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("named read test field keys should be foundational")
}

pub fn binding_label(target: &WorthServerQueryWorkspaceBindingTarget) -> Option<&str> {
    match target {
        worth_server::WorthServerQueryWorkspaceBindingTarget::DirectDeclaration {
            source_kind: worth_server::WorthServerDirectDeclarationSourceKind::NamedRead,
            binding_label,
        } => Some(binding_label),
        worth_server::WorthServerQueryWorkspaceBindingTarget::QueryHandoff { operation } => {
            query_handoff_binding_label(operation)
        }
        worth_server::WorthServerQueryWorkspaceBindingTarget::DirectDeclaration { .. } => None,
    }
}

pub fn query_handoff_binding_label(operation: &WorthServerQueryHandoffOperation) -> Option<&str> {
    match operation {
        worth_server::WorthServerQueryHandoffOperation::QueryRead { operation_name }
        | worth_server::WorthServerQueryHandoffOperation::DirectRead { operation_name } => Some(operation_name),
        worth_server::WorthServerQueryHandoffOperation::DirectState { target_label }
        | worth_server::WorthServerQueryHandoffOperation::DirectInspection { target_label }
        | worth_server::WorthServerQueryHandoffOperation::DirectProjection { target_label } => Some(target_label),
        worth_server::WorthServerQueryHandoffOperation::DirectMutation { .. }
        | worth_server::WorthServerQueryHandoffOperation::QueryMutation { .. }
        | worth_server::WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => None,
    }
}