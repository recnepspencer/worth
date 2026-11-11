//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOwnedAsyncRequestDeclaration>(
            "workspaces/worth-query/crates/worth-query/src/runtime/owned_async_source.rs",
            "WorthQueryOwnedAsyncRequestDeclaration",
            128 * 1024,
        );
        owner.field::<crate::application::WorthQueryAsyncResourceRequestIdentity>("identity");
        owner.field::<crate::application::WorthQueryAsyncDeclarationClause>("clause");
        owner.field::<u64>("payload_contract");
        owner.field::<u64>("max_payload_bytes");
        owner.field::<u32>("retry_max_attempts");
        owner.field::<u64>("retry_delay_ticks");
        owner.field::<u64>("timeout_ticks");
    }
}
