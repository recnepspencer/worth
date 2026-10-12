use worth_query::facade::foundation::AspectFieldKey;
use worth_query::facade::runtime::WorthQueryWorkspace;
use worth_server::{
    WorthServerDirectDeclarationSourceKind, WorthServerQueryHandoffOperation,
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceBindingTarget,
};

pub(super) fn install_requested_named_read(
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
            WorthServerQueryWorkspaceBindingError::new(
                "workspace_declaration",
                format!("{error:?}"),
            )
        })
}

fn aspect_field_key(aspect: &str, field: &str) -> AspectFieldKey {
    AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("named read test field keys should be foundational")
}

fn binding_label(target: &WorthServerQueryWorkspaceBindingTarget) -> Option<&str> {
    match target {
        WorthServerQueryWorkspaceBindingTarget::DirectDeclaration {
            source_kind: WorthServerDirectDeclarationSourceKind::NamedRead,
            binding_label,
        } => Some(binding_label),
        WorthServerQueryWorkspaceBindingTarget::QueryHandoff { operation } => {
            query_handoff_binding_label(operation)
        }
        WorthServerQueryWorkspaceBindingTarget::DirectDeclaration { .. } => None,
    }
}

fn query_handoff_binding_label(operation: &WorthServerQueryHandoffOperation) -> Option<&str> {
    match operation {
        WorthServerQueryHandoffOperation::QueryRead { operation_name }
        | WorthServerQueryHandoffOperation::DirectRead { operation_name } => Some(operation_name),
        WorthServerQueryHandoffOperation::DirectState { target_label }
        | WorthServerQueryHandoffOperation::DirectInspection { target_label }
        | WorthServerQueryHandoffOperation::DirectProjection { target_label } => Some(target_label),
        WorthServerQueryHandoffOperation::DirectMutation { .. }
        | WorthServerQueryHandoffOperation::QueryMutation { .. }
        | WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => None,
    }
}
