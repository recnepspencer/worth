use crate::domain_computation::primary_graph::tests::application_query::derived_view::leased_reconstruction::observation::*;

pub fn observe(
    workers: Option<usize>,
    count: usize,
    duplicate: bool,
    owner_failure: bool,
    failures: &[usize],
) -> Observation {
    observe_case(
        workers,
        count,
        duplicate,
        owner_failure,
        failures,
        Case::default(),
    )
}
