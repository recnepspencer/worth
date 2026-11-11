//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeAsyncRequestAdmissionRequest>(
            "crates/worth-runtime-bridge/src/source/async_declaration/request_identity/identity.rs",
            "BridgeAsyncRequestAdmissionRequest",
            128 * 1024,
        );
        owner.field::<LoweredBridgeAsyncSourceDeclaration>("lowered");
        owner.field::<ValidatedBridgeAsyncRequestBasisBinding>("basis_binding");
        owner.field::<BridgeAsyncRequestFamilyAdmission>("family_admission");
    }
}
