//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOperationAdvance<(), (), ()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/fixed.rs", "WorthQueryOperationAdvance", 128 * 1024);
        owner.field::<()>("Progress.operation");
        owner.field::<WorthQueryOperationProgress>("Progress.progress");
        owner.field::<()>("Refused.operation");
        owner.field::<WorthQueryOperationAdvanceDenial>("Refused.denial");
        owner.field::<()>("Terminal.operation");
        owner.field::<WorthQueryOperationAdvanceDenial>("Terminal.denial");
    }
}
