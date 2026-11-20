pub(crate) use worth_query_testkit_certification::facade::certification::runtime_world_branching::cumulative::*;

#[test]
fn sibling_writer_commits_while_combined_publication_is_parked() {
    cost::assert_sibling_writer_progress_while_combined_is_parked();
}

#[test]
fn graph_work_capacity_obeys_installed_exact_bound() {
    cost::assert_graph_work_capacity_bounds();
}
