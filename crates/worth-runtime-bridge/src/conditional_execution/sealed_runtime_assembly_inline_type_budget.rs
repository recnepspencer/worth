//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeSealedRuntimeAssembly>(
            "crates/worth-runtime-bridge/src/conditional_execution/sealed_runtime_assembly.rs",
            "BridgeSealedRuntimeAssembly",
            128 * 1024,
        );
        owner.field::<BridgeOwnedSignalRuntime>("runtime");
        owner.field::<worth_signal::facade::runtime::SignalConditionalEvaluationBudget>(
            "conditional_evaluation_budget",
        );
        owner.field::<SignalOwnerServicePorts<(), (), (), (), ()>>("signal_services");
        owner.field::<Option<
            worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
                (),
                (),
                (),
                (),
                (),
            >,
        >>("signal_definition_publication");
        owner.field::<worth_signal::facade::branch::AdmittedSignalBranchBasis>("signal_basis");
        owner.field::<Option<RuntimeWorldCorrespondencePort>>("correspondence");
        owner.field::<crate::facade::AdmittedRuntimeWorldCorrespondenceBasis>(
            "correspondence_basis",
        );
        owner.field::<BridgeSealedOwnerAffinity>("affinity");
        owner.field::<Option<home_retirement::FundedBridgeAssemblyRetirement>>("home_retirement");
    }
    {
        let mut owner = census.owner::<BridgeSealedOwnerAffinity>(
            "crates/worth-runtime-bridge/src/conditional_execution/sealed_runtime_assembly.rs",
            "BridgeSealedOwnerAffinity",
            128 * 1024,
        );
        owner.field::<u64>("signal_graph_instance_id");
    }
    super::home_retirement::inline_type_budget::collect(census);
}
