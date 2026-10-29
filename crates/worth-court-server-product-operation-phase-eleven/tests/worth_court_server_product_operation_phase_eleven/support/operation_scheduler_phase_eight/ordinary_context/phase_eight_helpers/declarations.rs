pub(crate) use {worth_query::facade::runtime::WorthQueryAspectMutationBuilder, worth_query::facade::runtime::WorthQueryWriteCommand};
pub(crate) use {worth_server::WorthServerRequestContextInput, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) fn worth_native_request_input() -> WorthServerRequestContextInput {
    worth_server::WorthServerRequestContextInput::builder()
        .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("request context input should validate")
}

pub(crate) fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}