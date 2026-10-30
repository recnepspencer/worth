#[allow(unused_imports)] use crate::signal_owner_services::ordinary_context::adversarial::operation_control::observation_race::*;

pub(crate) use crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::prove_retirement_is_effectful;

#[test]
fn same_branch_observe_retire_returns_a_complete_pre_or_post_state() {
    prove_retirement_is_effectful();
    run_order(true);
    run_order(false);
}
