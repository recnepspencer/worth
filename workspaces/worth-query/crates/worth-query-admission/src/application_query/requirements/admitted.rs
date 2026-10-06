use std::mem::size_of;

use worth_foundational::facade::{CanonicalDigestId, CanonicalDigestWorkBudget};
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryPlanningInventoryStop,
};

use super::WorthQueryApplicationQueryLane;
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
use crate::graph_read_access::{
    derive_canonical_graph_read_access_requirements_admitted,
    WorthQueryCanonicalGraphReadPlanningInput, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadPlanningIdentity,
};

mod shape;

pub fn derive_graph_read_access_requirements_for_contract_admitted<Stop>(
    graph: &WorthQueryInstalledGraphReadContract,
    lane: WorthQueryApplicationQueryLane,
    maximum_result_count: usize,
    selectivity_binding_digest: &CanonicalDigestId,
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQueryGraphReadAccessRequirementSet, WorthQueryCanonicalIdentityStop<Stop>> {
    admit(6, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let inventory = graph
        .admitted_planning_inventory(admit)
        .map_err(inventory_stop)?;
    let planning_graph_identity = *graph.canonical_planning_basis().digest();
    let (relations, has_many_relation) = shape::relations(&inventory, admit)?;
    let (access_shape_digest, access_work) = super::identity::access_shape_digest_admitted(
        graph,
        &inventory,
        planning_graph_identity,
        lane,
        maximum_result_count,
        budget,
        admit,
    )?;
    let (selectivity_shape_digest, selectivity_work) =
        super::identity::selectivity_shape_digest_admitted(
            planning_graph_identity,
            access_shape_digest,
            *selectivity_binding_digest,
            budget,
            admit,
        )?;
    let identity = WorthQueryGraphReadPlanningIdentity::from_admitted_evidence(
        planning_graph_identity,
        access_shape_digest,
        selectivity_shape_digest,
        *graph.schema_basis_digest(),
    );
    let shape = shape::finish(graph, &inventory, relations, has_many_relation, lane, admit)?;
    admit(3, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let input = WorthQueryCanonicalGraphReadPlanningInput::from_admitted_evidence(identity, shape)
        .with_maximum_cardinality(maximum_result_count)
        .with_live_maintenance_required(lane == WorthQueryApplicationQueryLane::Live);
    derive_canonical_graph_read_access_requirements_admitted(
        &input,
        budget,
        access_work.combine(selectivity_work),
        admit,
    )
}

fn inventory_stop<Stop>(
    stop: WorthQueryPlanningInventoryStop<Stop>,
) -> WorthQueryCanonicalIdentityStop<Stop> {
    match stop {
        WorthQueryPlanningInventoryStop::Admission(stop) => {
            WorthQueryCanonicalIdentityStop::Admission(stop)
        }
        WorthQueryPlanningInventoryStop::AccountingOverflow => {
            WorthQueryCanonicalIdentityStop::AccountingOverflow
        }
    }
}

fn reserve_vec<T, Stop>(
    count: usize,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<Vec<T>, WorthQueryCanonicalIdentityStop<Stop>> {
    use WorthQueryCanonicalIdentityStop::{AccountingOverflow, Admission, AllocationUnavailable};
    let bytes = count
        .checked_mul(size_of::<T>())
        .ok_or(AccountingOverflow)?;
    let bytes = u64::try_from(bytes).map_err(|_| AccountingOverflow)?;
    // Every initialized row is written into the admitted output backing.
    let work = bytes.checked_add(1).ok_or(AccountingOverflow)?;
    admit(work, bytes).map_err(Admission)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| AllocationUnavailable)?;
    Ok(rows)
}

fn copied_bytes<Stop>(parts: &[usize]) -> Result<u64, WorthQueryCanonicalIdentityStop<Stop>> {
    parts.iter().try_fold(0u64, |total, part| {
        total
            .checked_add(
                u64::try_from(*part)
                    .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?,
            )
            .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)
    })
}
