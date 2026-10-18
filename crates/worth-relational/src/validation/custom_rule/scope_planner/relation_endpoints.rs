//! Keep this registration's captured relation endpoints through scope restriction.
use super::super::CustomInvariantWorkMeter;
use crate::{
    identity::data::EntityId,
    validation::{
        data::{CustomInvariantAccessContract, TouchedStructuralSet},
        engine::state_view::InvariantStateView,
    },
};
pub(super) fn retains_entity(
    id: EntityId,
    touched: &TouchedStructuralSet,
    state: &InvariantStateView<'_>,
    before: &InvariantStateView<'_>,
    access: &CustomInvariantAccessContract,
    work: &CustomInvariantWorkMeter,
) -> bool {
    let comparisons = access
        .affected_entity_kinds
        .len()
        .saturating_add(access.read_entity_kinds.len())
        .saturating_mul(2)
        .saturating_add(2);
    if !work.try_charge_work(u64::try_from(comparisons).unwrap_or(u64::MAX)) {
        return false;
    }
    let metadata = [state.entity_metadata(id), before.entity_metadata(id)];
    if metadata
        .iter()
        .flatten()
        .any(|record| access.affects_entity(record.kind_id))
    {
        return true;
    }
    if access.affected_relation_kinds.is_empty()
        || !metadata
            .iter()
            .flatten()
            .any(|record| access.reads_entity(record.kind_id))
    {
        return false;
    }
    // Capture is already per-registration (packet_selection), using this SAME
    // access contract. A direct ID of a non-affected entity can therefore only
    // be an actual old/new endpoint of its affected relations. Readability alone
    // is insufficient: never retain adjacency-only neighbors as direct context.
    if !work.try_charge_work(
        u64::try_from(touched.direct_visible_entity_ids().len()).unwrap_or(u64::MAX),
    ) {
        return false;
    }
    touched.direct_visible_entity_ids().contains(&id)
}
