use super::*;

pub(in super::super) fn observe(
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
