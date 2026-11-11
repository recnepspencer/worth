//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RelationalBridgeRecordIdentityParts>(
            "crates/worth-runtime-bridge/src/relational_source/identity_parts.rs",
            "RelationalBridgeRecordIdentityParts",
            128 * 1024,
        );
        owner.field::<RelationalBridgeRecordIdentityKind>("kind");
        owner.field::<u32>("partition_id");
        owner.field::<u64>("local_slot");
        owner.field::<u32>("generation");
    }
}
