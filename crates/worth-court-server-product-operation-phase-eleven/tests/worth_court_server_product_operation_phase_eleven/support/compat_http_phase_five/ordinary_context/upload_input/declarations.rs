pub(crate) use {
    worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerCompatibilityRequestInput,
};

pub(crate) fn upload_input_with_content_type(
    operation_name: &str,
    content_type: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type(content_type)
        .with_body_present(true)
}
