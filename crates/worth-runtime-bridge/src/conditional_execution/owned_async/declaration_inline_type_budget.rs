//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeOwnedAsyncRequestResponseDeclaration>(
            "crates/worth-runtime-bridge/src/conditional_execution/owned_async/declaration.rs",
            "BridgeOwnedAsyncRequestResponseDeclaration",
            128 * 1024,
        );
        owner.field::<Arc<str>>("identity");
        owner.field::<u64>("payload_contract");
        owner.field::<u64>("max_payload_bytes");
        owner.field::<u32>("retry_max_attempts");
        owner.field::<u64>("retry_delay_ticks");
        owner.field::<u64>("timeout_ticks");
        owner.field::<Option<std::num::NonZeroU64>>("managed_queue_capacity");
        owner.field::<bool>("process_local_retention");
    }
}
