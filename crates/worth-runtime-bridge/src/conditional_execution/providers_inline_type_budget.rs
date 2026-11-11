//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalProviderSet>(
            "crates/worth-runtime-bridge/src/conditional_execution/providers.rs",
            "BridgeConditionalProviderSet",
            128 * 1024,
        );
        owner.field::<Option<Arc<dyn BridgeConditionalConditionProvider>>>("condition");
        owner
            .field::<Option<Arc<dyn BridgeConditionalComparatorProvider>>>("dependency_comparator");
        owner.field::<Option<Arc<dyn BridgeConditionalComparatorProvider>>>("output_comparator");
        owner.field::<Option<Arc<dyn BridgeConditionalComparatorProvider>>>("reuse_comparator");
        owner.field::<Option<Arc<dyn BridgeConditionalTriggerProvider>>>("trigger");
        owner.field::<Option<Arc<dyn BridgeConditionalWakeProvider>>>("wake");
        owner.field::<Option<Arc<dyn BridgeConditionalComputeProvider>>>("compute");
        owner.field::<BridgeConditionalProviderSemanticContracts>("semantic_contracts");
    }
}
