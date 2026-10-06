use super::{
    worth_query_graph_index_inventory, WorthQueryGraphIndexInventory,
    WorthQueryGraphIndexInventoryMatchReport,
};
use crate::graph_read_access::digest_text::AdmittedDigestTextStop;
use crate::graph_read_access::WorthQueryGraphReadAccessRequirementSet;

pub fn match_graph_index_inventory_for_requirements(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    inventory: &WorthQueryGraphIndexInventory,
) -> WorthQueryGraphIndexInventoryMatchReport {
    WorthQueryGraphIndexInventoryMatchReport::match_requirements(requirements, inventory)
}

pub(crate) fn match_graph_index_inventory_for_requirements_admitted<Stop>(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    inventory: &WorthQueryGraphIndexInventory,
    admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphIndexInventoryMatchReport, AdmittedDigestTextStop<Stop>> {
    WorthQueryGraphIndexInventoryMatchReport::match_requirements_admitted(
        requirements,
        inventory,
        admit,
    )
}

pub fn match_current_graph_index_inventory_for_requirements(
    requirements: &WorthQueryGraphReadAccessRequirementSet,
) -> WorthQueryGraphIndexInventoryMatchReport {
    let inventory = worth_query_graph_index_inventory();
    WorthQueryGraphIndexInventoryMatchReport::match_requirements(requirements, &inventory)
}
