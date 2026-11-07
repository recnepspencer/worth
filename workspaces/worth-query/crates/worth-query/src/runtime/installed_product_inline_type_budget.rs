//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryInstalledProduct>(
            "workspaces/worth-query/crates/worth-query/src/runtime/installed_product.rs",
            "WorthQueryInstalledProduct",
            128 * 1024,
        );
        owner.field::<WorthQueryProductRuntime>("world");
        owner.field::<BridgeSealedRuntimeAssembly>("conditional");
        owner.field::<worth_runtime_bridge::facade::RuntimeBridge>("managed_execution_bridge");
        owner.field::<conditional_evaluation::WorthQueryConditionalEvaluationRegistry>(
            "conditional_evaluations",
        );
    }
}
