//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryProductUnpublishedApplication>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/product_publication/unpublished.rs", "WorthQueryProductUnpublishedApplication", 128 * 1024);
        owner.field::<WorthQueryProductUnpublishedOwnerEffects>("effects");
        owner.field::<RuntimeWorldRecoveryPort>("recovery");
        owner.field::<crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition>("disposition");
    }
}
