use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::{
    effect_lifecycle_support_matrix, effect_lifecycle_support_row_matches_inventory,
    EffectDeferredNeighborFamily, EffectFamily,
};

#[test]
fn support_rows_match_the_family_inventory_contract() {
    let matrix = effect_lifecycle_support_matrix();

    assert!(matrix
        .rows()
        .iter()
        .all(effect_lifecycle_support_row_matches_inventory));
}

#[test]
fn support_rows_expose_denial_rebind_and_deferred_neighbor_contracts() {
    let matrix = effect_lifecycle_support_matrix();

    let preview_mutation = matrix
        .rows()
        .iter()
        .find(|row| {
            row.basis_family() == BasisFamily::Preview
                && row.effect_family() == EffectFamily::Mutation
        })
        .expect("preview mutation support row should exist");
    assert!(preview_mutation.requires_rebind());
    assert_eq!(
        preview_mutation.denial_kinds(),
        &[crate::effect_lifecycle::DeniedEffectEligibilityKind::PreviewRebindRequired]
    );

    let current_writeback = matrix
        .rows()
        .iter()
        .find(|row| {
            row.basis_family() == BasisFamily::CurrentHead
                && row.effect_family() == EffectFamily::Writeback
        })
        .expect("current-head writeback support row should exist");
    assert_eq!(
        current_writeback.deferred_neighbors(),
        &[
            EffectDeferredNeighborFamily::StoreBackedExecutionParity,
            EffectDeferredNeighborFamily::DurableReplayAndRestartStableEnvelope,
        ]
    );
}
