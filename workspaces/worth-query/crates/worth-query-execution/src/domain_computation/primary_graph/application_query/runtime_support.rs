use std::convert::Infallible;

use worth_query_admission::facade::graph_read_access::{
    WorthQueryGraphIndexInventory, WorthQueryGraphReadAccessRequirementSet,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationContinuationContract, WorthQueryInstalledApplicationLiveContract,
};

use super::super::schema_layout::WorthQueryPrimaryGraphLayout;

mod admitted;

pub(in crate::domain_computation::primary_graph) use admitted::primary_graph_support_inventory_admitted;

pub(in crate::domain_computation::primary_graph) fn primary_graph_support_inventory(
    layout: &WorthQueryPrimaryGraphLayout,
    continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    live: Option<&WorthQueryInstalledApplicationLiveContract>,
    requirements: &WorthQueryGraphReadAccessRequirementSet,
) -> WorthQueryGraphIndexInventory {
    primary_graph_support_inventory_admitted(
        layout,
        continuation,
        live,
        requirements,
        &mut |_, _| Ok::<(), Infallible>(()),
    )
    .expect("ordinary support inventory has no resource refusal")
}
