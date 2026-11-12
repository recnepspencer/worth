//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalInstallationRequest>(
            "crates/worth-runtime-bridge/src/conditional_execution/contract.rs",
            "BridgeConditionalInstallationRequest",
            128 * 1024,
        );
        owner.field::<super::super::BridgeConditionalContract>("contract");
        owner.field::<super::super::BridgeConditionalLocation>("location");
        owner.field::<Vec<crate::correspondence::BridgeSemanticCorrespondenceRegistration>>(
            "registrations",
        );
        owner.field::<BridgeConditionalProviderSet>("providers");
    }
    {
        let mut owner = census.owner::<AdmittedConditionalInstallationRequest>(
            "crates/worth-runtime-bridge/src/conditional_execution/contract.rs",
            "AdmittedConditionalInstallationRequest",
            128 * 1024,
        );
        owner.field::<BridgeConditionalInstallationRequest>("request");
        owner.field::<super::super::provider_admission::BridgeConditionalProviderAdmission>(
            "provider_admission",
        );
        owner.field::<Option<worth_signal::facade::NodeId>>("node");
        owner.field::<worth_signal::facade::branch::SignalBranchIdentity>("signal_branch_identity");
        owner.field::<crate::correspondence::AdmittedSemanticDependencyExtension>(
            "dependency_extension",
        );
        owner.field::<Option<
            super::super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan,
        >>("semantic_observation_plan");
        owner.field::<BridgeInstalledConditionalLoweringCounters>("counters");
    }
    {
        let mut owner = census.owner::<BridgeOwnedSignalRuntime>(
            "crates/worth-runtime-bridge/src/conditional_execution/contract.rs",
            "BridgeOwnedSignalRuntime",
            128 * 1024,
        );
        owner.field::<OrdinaryBridgeCustody>("bridge");
        owner.field::<Arc<super::super::retention::BridgeRetentionLedger>>("retention");
        owner.field::<usize>("baseline_semantic_dependency_count");
        owner.field::<Mutex<Option<SignalRuntime<(), (), (), (), ()>>>>("signal_runtime");
        owner.field::<u64>("signal_graph_instance_id");
        owner.field::<worth_signal::facade::SignalGraphLifecycleProbe>(
            "signal_graph_lifecycle_probe",
        );
        owner.field::<Option<super::super::service_binding::BridgeSignalServiceBinding>>(
            "signal_services",
        );
        owner.field::<RwLock<
            BTreeMap<Arc<str>, super::super::owned_async::BridgeInstalledOwnedAsyncDeclaration>,
        >>("async_declarations");
        owner.field::<Arc<()>>("async_observation_authority");
        owner.field::<Arc<RwLock<super::super::lowering_registry::BridgeConditionalLoweringRegistry>>>("conditional_lowerings");
        owner.field::<Arc<RwLock<super::super::owned_target_index::BridgeOwnedConditionalTargetIndex>>>("owned_conditional_targets");
        owner.field::<std::sync::Mutex<
            BTreeMap<Arc<str>, Arc<Mutex<super::super::managed_time::BridgeManagedClockLane>>>,
        >>("managed_clock_lanes");
        owner.field::<std::sync::atomic::AtomicU64>("next_owned_semantic_publication");
    }
    super::home_retirement_slots::inline_type_budget::collect(census);
}
