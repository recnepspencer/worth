use worth_query::facade::runtime::{WorthQueryAspectMutationBuilder, WorthQueryWriteCommand};
use worth_server::{
    WorthServerRequestContextInput, WorthServerSurfaceFamily, WorthServerTransportClass,
};

pub(crate) fn worth_native_request_input() -> WorthServerRequestContextInput {
    WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .build()
        .expect("request context input should validate")
}

pub(crate) fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}
