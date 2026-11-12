//! Keep graph completion metadata backed while Signal owner services are sealed.
use super::*;
use crate::domain_computation::primary_graph::bootstrap::{
    BootstrapPublicationReservation, BootstrapPublicationStorage,
};
type Unsealed = super::super::super::managed_bridge::WorthQueryApplicationBridgeInstallation;
type Sealed = super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge;
struct GraphCompletion {
    runtime: WorthQueryExecutionRuntime,
    publication: super::super::WorthQueryPrimaryGraphPublication,
    relational_branch_identity: worth_relational::facade::branch::RelationalBranchIdentity,
    primary_provider: std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    primary_graph_authority: WorthQueryInstalledGraphParticipationAuthority,
    product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    recovered_relational_authority: Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    checkpoint_restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
}
#[inline(never)]
fn prepare(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    graph: BootstrapPublicationStorage<PublishedApplicationGraph<Unsealed>>,
) -> Result<
    (
        BootstrapPublicationStorage<Unsealed>,
        BootstrapPublicationStorage<GraphCompletion>,
    ),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let bridge_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let completion_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge,
        primary_provider,
        primary_graph_authority,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    } = graph.consume();
    Ok((
        bridge_storage.fill(bridge),
        completion_storage.fill(GraphCompletion {
            runtime,
            publication,
            relational_branch_identity,
            primary_provider,
            primary_graph_authority,
            product_world_resources,
            recovered_relational_authority,
            checkpoint_restore_work,
        }),
    ))
}
#[inline(never)]
fn seal_bridge(
    storage: BootstrapPublicationReservation<Sealed>,
    bridge: BootstrapPublicationStorage<Unsealed>,
) -> Result<BootstrapPublicationStorage<Sealed>, WorthQueryPrimaryGraphInstallationDenial> {
    let bridge = bridge.consume().seal().map_err(|denial| {
        WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
            format!("{denial:?}"),
        )
        .with_bridge_conditional_cause(denial)
    })?;
    Ok(storage.fill(bridge))
}
#[inline(never)]
fn finish(
    storage: BootstrapPublicationReservation<PublishedApplicationGraph<Sealed>>,
    bridge: BootstrapPublicationStorage<Sealed>,
    completion: BootstrapPublicationStorage<GraphCompletion>,
) -> BootstrapPublicationStorage<PublishedApplicationGraph<Sealed>> {
    let GraphCompletion {
        runtime,
        publication,
        relational_branch_identity,
        primary_provider,
        primary_graph_authority,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    } = completion.consume();
    storage.fill(PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge: bridge.consume(),
        primary_provider,
        primary_graph_authority,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    })
}
#[inline(never)]
pub(super) fn seal(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    graph: BootstrapPublicationStorage<PublishedApplicationGraph<Unsealed>>,
) -> Result<
    BootstrapPublicationStorage<PublishedApplicationGraph<Sealed>>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let bridge_storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let (bridge, completion) = prepare(phase, graph)?;
    let bridge = seal_bridge(bridge_storage, bridge)?;
    Ok(finish(storage, bridge, completion))
}
