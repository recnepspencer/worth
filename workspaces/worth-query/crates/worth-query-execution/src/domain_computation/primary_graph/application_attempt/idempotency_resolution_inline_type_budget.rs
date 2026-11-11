//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryApplicationIdempotencyResolution>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/idempotency_resolution.rs", "WorthQueryApplicationIdempotencyResolution", 128 * 1024);
    }
}
