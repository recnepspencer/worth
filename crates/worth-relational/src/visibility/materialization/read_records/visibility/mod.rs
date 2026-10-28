mod history;
mod kind_matching;
mod lifecycle;
mod slot_sets;

pub(super) use history::{historical_created_at, visible_metadata, visible_relation_metadata};
pub(super) use kind_matching::{
    entity_slot_matches_kind_at_version, relation_slot_matches_kind_at_version,
};
pub(super) use lifecycle::{
    historical_lifecycle, historical_relation_lifecycle, historical_retired_at,
    lifecycle_storage_visible,
};
pub(super) use slot_sets::{
    entity_visible_in_partition_at_version, relation_visible_in_partition_at_version,
    visible_relation_slots_in_partition_from_state, visible_slots_in_partition_from_state,
};

#[cfg(test)]
mod tests;
