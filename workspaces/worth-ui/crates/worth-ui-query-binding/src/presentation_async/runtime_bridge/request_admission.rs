use worth_query::facade::runtime;

pub(super) fn admit_presentation_owned_request(
    workspace: &runtime::WorthQueryWorkspace,
    declaration: &runtime::WorthQueryInstalledOwnedAsyncDeclaration,
    product: &runtime::WorthQueryProductBranchLease,
) -> Result<
    worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    runtime::WorthQueryOwnedAsyncRuntimeDenial,
> {
    workspace.admit_owned_bridge_async_request(declaration, product)
}
