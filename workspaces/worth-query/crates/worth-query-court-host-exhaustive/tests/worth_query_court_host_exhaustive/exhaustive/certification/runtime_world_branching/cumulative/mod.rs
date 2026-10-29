pub(crate) use worth_query_worlds::facade::certification::runtime_world_branching::cumulative::*;

#[test]
fn seeded_model_checks_public_branch_action_roster() {
    model::assert_seeded_public_action_roster();
}
