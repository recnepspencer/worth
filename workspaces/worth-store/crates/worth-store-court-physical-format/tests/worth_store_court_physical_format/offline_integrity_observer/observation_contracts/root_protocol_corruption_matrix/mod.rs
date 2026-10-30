#[allow(unused_imports)]
use crate::root_protocol_corruption_matrix::*;

#[test]
fn current_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Current);
}

#[test]
fn previous_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Previous);
}

#[test]
fn addressed_root_manifest_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(crate::store_offline_integrity_observer_support::Target::Root);
}
