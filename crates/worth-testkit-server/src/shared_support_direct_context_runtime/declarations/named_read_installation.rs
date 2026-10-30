use super::*;

pub fn install_requested_named_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let worth_server::WorthServerQueryWorkspaceBindingTarget::DirectDeclaration {
        source_kind: worth_server::WorthServerDirectDeclarationSourceKind::NamedRead,
        binding_label,
    } = request.target()
    else {
        return Ok(());
    };

    workspace
        .live_view::<serde_json::Value>(request_execution, binding_label, |q| {
            q.from("User")
                .select([
                    aspect_field_key("identity", "id"),
                    aspect_field_key("profile", "display_name"),
                ])
                .schema_basis("worth-server-direct-context-remask")
        })
        .map(|_| ())
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new(
                "workspace_declaration",
                format!("{error:?}"),
            )
        })
}

pub fn aspect_field_key(
    aspect: &str,
    field: &str,
) -> worth_query::facade::foundation::AspectFieldKey {
    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("direct context field keys should be foundational")
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    let fields = path
        .split('.')
        .map(|field| {
            worth_foundational::facade::FieldKey::new(field)
                .expect("direct context field segments should be foundational")
        })
        .collect::<Vec<_>>();
    worth_foundational::facade::CanonicalFieldPath::new(fields)
        .expect("direct context field path should be non-empty")
}
