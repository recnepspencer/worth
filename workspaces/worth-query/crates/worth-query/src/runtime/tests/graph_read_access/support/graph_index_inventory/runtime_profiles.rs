use crate::runtime::{
    WorthQueryGraphReadAccessRequirementKind, WorthQueryRuntimeSupportProfile, WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support::public_bridge_runtime::{
    public_graph_support_profile, PublicBridgeRuntimeHarness,
};

pub fn workspace_with_graph_support(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    support_profile: WorthQueryRuntimeSupportProfile,
) -> WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime_with_support(request_execution, support_profile)
        .workspace(workspace_name)
        .expect("runtime should open graph inventory workspace")
}

pub fn default_graph_support_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
) -> WorthQueryWorkspace {
    workspace_with_graph_support(
        request_execution,
        workspace_name,
        public_graph_support_profile(),
    )
}

pub fn profile_without_graph_support(
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_graph_index_support_omitted(requirement_kind)
}

pub fn profile_with_graph_support_temporarily_unavailable(
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_graph_index_temporarily_unavailable(requirement_kind)
}

pub fn profile_with_ephemeral_graph_support(
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_graph_index_ephemeral_available(requirement_kind)
}

pub fn profile_requiring_graph_access_capability_registration(
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_graph_index_access_capability_registration_required(
        requirement_kind,
        "worth-query-9.10-test-capability-registration",
    )
}

pub fn profile_requiring_store_backed_graph_index(
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
) -> WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_store_backed_graph_index_requirement(
        requirement_kind,
        "worth-query-9.10-test-store-backed-index",
    )
}
