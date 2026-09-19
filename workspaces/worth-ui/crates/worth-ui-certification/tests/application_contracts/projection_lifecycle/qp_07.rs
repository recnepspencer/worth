#[test]
fn every_rebind_safe_point_disposes_or_returns_its_exact_owner() {
    crate::observation_rebind::lifecycle_cleanup::prove_all_projection_safe_point_cleanup();
}

#[test]
fn collection_continuation_reset_and_hostile_denials_close_every_owner() {
    crate::collection_projection::qp_04::mutations::
        complete_partial_and_continuation_postures_come_from_real_query_results();
    crate::collection_projection::qp_04::mutations::
        continuation_completion_and_explicit_reset_are_preserved();
    crate::collection_projection::qp_04::hostile::
        hostile_query_patches_return_exact_denials_and_mint_no_ui_effect();
}
