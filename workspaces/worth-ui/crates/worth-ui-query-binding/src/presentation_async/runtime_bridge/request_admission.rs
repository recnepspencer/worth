use worth_query::facade::runtime;

pub(super) fn admit_presentation_owned_request(
    workspace: &runtime::WorthQueryWorkspace,
    declaration: &runtime::WorthQueryInstalledOwnedAsyncDeclaration,
    product: &worth_query::facade::product::WorthQueryProductBranchLease,
) -> Result<runtime::WorthQueryOwnedAsyncRequestCustody, runtime::WorthQueryOwnedAsyncRuntimeStop> {
    workspace.admit_owned_bridge_async_request_at_branch_head(declaration, product)
}
