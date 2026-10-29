pub(crate) use worth_query_worlds::facade::certification::runtime_world_branching::cumulative::*;

#[test]
#[ignore = "scheduled broader seeded model; ordinary CI runs the bounded action roster"]
fn scheduled_seeded_model_broadens_public_action_roster() {
    model::assert_scheduled_public_action_roster();
}

#[test]
#[ignore = "scheduled timing evidence; structural counters are the ordinary regression gate"]
fn scheduled_public_read_timings_report_cold_and_warm_distributions() {
    cost::run_scheduled_public_read_timings();
}
