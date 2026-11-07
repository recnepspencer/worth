//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::request_identity::inline_type_budget::collect(census);
    let _owner = census.owner::<super::LoweredBridgeAsyncSourceDeclaration>(
        "crates/worth-runtime-bridge/src/source/async_declaration/lowering.rs",
        "LoweredBridgeAsyncSourceDeclaration",
        256,
    );
}
