use super::{WorthUiPlanRegionMutation, WorthUiPlanRegionStore};

pub(super) fn expected_region_count_after(
    store: &WorthUiPlanRegionStore,
    mutation: &WorthUiPlanRegionMutation,
    current: usize,
) -> usize {
    let exists = store.handle_for(mutation.identity()).is_some();
    match mutation {
        WorthUiPlanRegionMutation::Insert(_) if !exists => current + 1,
        WorthUiPlanRegionMutation::Upsert(_) if !exists => current + 1,
        WorthUiPlanRegionMutation::Retire(_) if exists => current - 1,
        WorthUiPlanRegionMutation::OwnerBundle { root, schemas } => {
            let predecessor_width = store
                .record_for_identity(root)
                .map(|record| record.executable.owned_region_identities().len() + 1)
                .unwrap_or_default();
            current - predecessor_width + schemas.len()
        }
        WorthUiPlanRegionMutation::RetireOwner(root) if exists => {
            let predecessor_width = store
                .record_for_identity(root)
                .map(|record| record.executable.owned_region_identities().len() + 1)
                .unwrap_or(1);
            current - predecessor_width
        }
        WorthUiPlanRegionMutation::Upsert(_)
        | WorthUiPlanRegionMutation::Insert(_)
        | WorthUiPlanRegionMutation::Replace(_)
        | WorthUiPlanRegionMutation::Reparent(_)
        | WorthUiPlanRegionMutation::Rebind(_)
        | WorthUiPlanRegionMutation::LaneTransition(_)
        | WorthUiPlanRegionMutation::Retire(_)
        | WorthUiPlanRegionMutation::RetireOwner(_) => current,
    }
}
