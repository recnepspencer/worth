pub use worth_query::facade::{consumer_kit, domain};

pub use super::installation::conditional_installation_pair_in_partitions;
pub use super::providers::DirectConditionalCompute;
pub use super::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadFamily, ReadVertex,
};

pub fn conditional_controlled_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<consumer_kit::WorthQueryControlledTestWorkspace, consumer_kit::WorthQueryTestBackendError>
{
    let installation = conditional_installation(&node);
    super::conditional_workspace_with_builder(node, installation, DirectConditionalCompute)
        .controlled_workspace(request_execution, name)
}

pub struct ConditionalWorkspacePlacement<'a> {
    pub name: &'a str,
    pub partition: &'a str,
}

pub struct ConditionalDonorWorkspaceScenario<'a, P> {
    pub owner: ConditionalWorkspacePlacement<'a>,
    pub donor: ConditionalWorkspacePlacement<'a>,
    pub node: domain::WorthQueryPortableConditionalNodeDeclaration,
    pub donor_compute: P,
}

pub fn conditional_controlled_workspace_with_donor<P>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    scenario: ConditionalDonorWorkspaceScenario<'_, P>,
) -> Result<
    (
        consumer_kit::WorthQueryControlledTestWorkspace,
        worth_query::facade::runtime::WorthQueryWorkspace,
    ),
    consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>,
{
    let ConditionalDonorWorkspaceScenario {
        owner: owner_placement,
        donor: donor_placement,
        node,
        donor_compute,
    } = scenario;
    let (owner_installation, donor_installation) = conditional_installation_pair_in_partitions(
        &node,
        owner_placement.partition,
        donor_placement.partition,
    );
    let owner = super::conditional_workspace_with_builder(
        node.clone(),
        owner_installation,
        DirectConditionalCompute,
    )
    .controlled_workspace(request_execution, owner_placement.name)?;
    let donor = conditional_workspace_with(
        request_execution,
        donor_placement.name,
        node,
        donor_installation,
        donor_compute,
    )?;
    Ok((owner, donor))
}
