use worth_query::facade::{consumer_kit, domain};

use super::installation::conditional_installation_pair_in_partitions;
use super::providers::DirectConditionalCompute;
use super::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadFamily, ReadVertex,
};

pub(crate) fn conditional_controlled_workspace(
    name: &str,
    node: domain::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<consumer_kit::WorthQueryControlledTestWorkspace, consumer_kit::WorthQueryTestBackendError>
{
    let installation = conditional_installation(&node);
    super::conditional_workspace_with_builder(node, installation, DirectConditionalCompute)
        .controlled_workspace(name)
}

pub(crate) struct ConditionalWorkspacePlacement<'a> {
    pub(crate) name: &'a str,
    pub(crate) partition: &'a str,
}

pub(crate) struct ConditionalDonorWorkspaceScenario<'a, P> {
    pub(crate) owner: ConditionalWorkspacePlacement<'a>,
    pub(crate) donor: ConditionalWorkspacePlacement<'a>,
    pub(crate) node: domain::WorthQueryPortableConditionalNodeDeclaration,
    pub(crate) donor_compute: P,
}

pub(crate) fn conditional_controlled_workspace_with_donor<P>(
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
    .controlled_workspace(owner_placement.name)?;
    let donor = conditional_workspace_with(
        donor_placement.name,
        node,
        donor_installation,
        donor_compute,
    )?;
    Ok((owner, donor))
}
