//! The fixture's refused public read owns two exact query-name strings.
use super::*;
pub(super) fn bytes(query: &str) -> u64 {
    use std::{mem::size_of, sync::OnceLock};
    // Payload fields all align to usize: optional authorization Box, two
    // Strings, and the sole reservation cell; Arc adds two reference counters.
    (size_of::<
        Option<
            Box<crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial>,
        >,
    >() + 2 * size_of::<String>()
        + size_of::<OnceLock<worth_execution::ExecutionMemoryReservation>>()
        + 2 * size_of::<usize>()
        + 2 * query.len()) as u64
}
pub(super) fn check(error: &WorthQueryManagedDerivedViewDenial, query: &str) {
    let WorthQueryManagedDerivedViewDenial::ReadDenied { denial, .. } = error else {
        unreachable!()
    };
    assert_eq!(denial.query(), query);
    assert_eq!(denial.subject(), query);
    assert!(
        denial.authorization_denial().is_none(),
        "the fixture chose public queries"
    );
}
