#[allow(unused_imports)] use crate::signal_owner_services::reference_context::adversarial::operation_control::ordered_races::*;
pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld};
pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::operation_control::ordered_race::run};
pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::advance, crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::capture, crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::prove_advance_is_effectful, crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::prove_capture_and_restore_are_effectful, crate::signal_owner_services::ordinary_context::adversarial::operation_control::races::restore};

#[test]
fn every_independently_admissible_same_branch_pair_forces_both_legal_winner_orderings() {
    prove_advance_is_effectful();
    prove_capture_and_restore_are_effectful();
    for left_parked in [true, false] {
        run_advance_advance(left_parked);
        run_advance_restore(left_parked);
        run_restore_restore(left_parked);
        run_snapshot_advance(left_parked);
    }
}
