use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock},
};

use worth_proof::TransitionOutcome;
use worth_signal::facade::{SignalGraph, SignalRuntime};

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalProviderSet,
    BridgeInstalledConditionalLowering, BridgeInstalledConditionalLoweringCounters,
};
use crate::facade::RuntimeBridge;

pub struct BridgeConditionalInstallationRequest {
    pub contract: super::BridgeConditionalContract,
    pub location: super::BridgeConditionalLocation,
    pub registrations: Vec<crate::correspondence::BridgeSemanticCorrespondenceRegistration>,
    pub providers: BridgeConditionalProviderSet,
}

pub(super) struct AdmittedConditionalInstallationRequest {
    pub(super) request: BridgeConditionalInstallationRequest,
    pub(super) provider_admission: super::provider_admission::BridgeConditionalProviderAdmission,
    pub(super) node: Option<worth_signal::facade::NodeId>,
    pub(super) signal_branch_identity: worth_signal::facade::branch::SignalBranchIdentity,
    pub(super) dependency_extension: crate::correspondence::AdmittedSemanticDependencyExtension,
    pub(super) semantic_observation_plan:
        Option<super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan>,
    pub(super) counters: BridgeInstalledConditionalLoweringCounters,
}

/// Bridge-owned ordinary conditional runtime. It retains the exact Signal
/// graph; Query never receives a raw graph or a detached node capability.
pub struct BridgeOwnedSignalRuntime {
    pub(super) bridge: RuntimeBridge,
    pub(super) retention: Arc<super::retention::BridgeRetentionLedger>,
    pub(super) baseline_semantic_dependency_count: usize,
    pub(super) signal_runtime: Mutex<SignalRuntime<(), (), (), (), ()>>,
    pub(super) signal_graph_instance_id: u64,
    pub(super) signal_graph_lifecycle_probe: worth_signal::facade::SignalGraphLifecycleProbe,
    pub(super) signal_services: Option<super::service_binding::BridgeSignalServiceBinding>,
    pub(super) async_declarations:
        RwLock<BTreeMap<Arc<str>, crate::facade::LoweredBridgeAsyncSourceDeclaration>>,
    pub(super) async_observation_authority: Arc<()>,
    pub(super) conditional_lowerings:
        Arc<RwLock<super::lowering_registry::BridgeConditionalLoweringRegistry>>,
    pub(super) owned_conditional_targets:
        Arc<RwLock<super::owned_target_index::BridgeOwnedConditionalTargetIndex>>,
    pub(super) managed_clock_lanes: std::sync::Mutex<
        BTreeMap<Arc<str>, Arc<Mutex<super::managed_time::BridgeManagedClockLane>>>,
    >,
    pub(super) next_owned_semantic_publication: std::sync::atomic::AtomicU64,
}

impl BridgeOwnedSignalRuntime {
    pub fn owned_signal_graph_instance_id(&self) -> u64 {
        self.signal_graph_instance_id
    }

    /// Owns a fresh Signal graph behind the Bridge boundary.
    ///
    /// Callers that do not already own a topology-specific Signal graph use
    /// this constructor so raw Signal authority never crosses into them.
    pub(super) fn with_owned_signal_graph(
        bridge: RuntimeBridge,
        evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<Self, BridgeConditionalDenial> {
        Self::new(bridge, Box::new(SignalGraph::new()), evaluation_budget)
    }

    pub(super) fn new(
        mut bridge: RuntimeBridge,
        mut graph: Box<SignalGraph>,
        evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<Self, BridgeConditionalDenial> {
        let retention =
            super::retention::BridgeRetentionLedger::new(bridge.policy.conditional_retention())
                .map_err(super::observation_retention::retention_denial)?;
        let signal_graph_instance_id = graph.installed_graph_capability().graph_instance_id();
        let signal_graph_lifecycle_probe = graph.lifecycle_probe();
        let baseline_semantic_dependency_count =
            bridge.semantic_dependency_registry.authoritative_count();
        crate::correspondence::isolate_allocation_state(&mut bridge).map_err(|_| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::CorrespondenceAdmission,
                "conditional runtime could not isolate its authoritative allocation state",
            )
        })?;
        graph
            .claim_aspect_lowering_owner(&bridge.signal_aspect_lowering_owner)
            .map_err(|_| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::ForeignSignalGraph,
                    "Signal graph already belongs to another lowering owner",
                )
            })?;
        let signal_runtime = SignalRuntime::builder_from_boxed_graph(graph);
        let signal_runtime = signal_runtime.runtime_policy(
            worth_signal::facade::runtime::SignalRuntimePolicy::development()
                .with_conditional_evaluation_budget(evaluation_budget),
        );
        let signal_runtime = signal_runtime.with_kernel_defaults();
        let signal_runtime = signal_runtime.build_validated().map_err(|error| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                format!("Signal conditional evaluation policy was denied: {error:?}"),
            )
        })?;
        Ok(Self {
            retention,
            bridge,
            baseline_semantic_dependency_count,
            signal_runtime: Mutex::new(signal_runtime),
            signal_graph_instance_id,
            signal_graph_lifecycle_probe,
            signal_services: None,
            async_declarations: Default::default(),
            async_observation_authority: Arc::new(()),
            conditional_lowerings: Arc::new(RwLock::new(Default::default())),
            owned_conditional_targets: Arc::new(RwLock::new(Default::default())),
            managed_clock_lanes: std::sync::Mutex::new(BTreeMap::new()),
            next_owned_semantic_publication: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub(super) fn signal_runtime_mut(&mut self) -> &mut SignalRuntime<(), (), (), (), ()> {
        self.signal_runtime
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn lock_signal_runtime(&self) -> MutexGuard<'_, SignalRuntime<(), (), (), (), ()>> {
        self.signal_runtime
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn install(
        &mut self,
        request: BridgeConditionalInstallationRequest,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        self.install_at_node(request, None)
    }

    pub(super) fn install_at_node(
        &mut self,
        request: BridgeConditionalInstallationRequest,
        owned_node: Option<worth_signal::facade::NodeId>,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        if self.signal_services.is_some() {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalContractInstallation,
                "sealed Signal definition requires the owner installation-extension operation",
            ));
        }
        let admitted = self.admit_conditional_installation_request(request, owned_node)?;
        let mut counters = admitted.counters;
        counters.signal_node_admissions += 1;
        let TransitionOutcome::Success(node_capability) =
            self.signal_runtime_mut().graph_mut().admit_installed_node(
                admitted
                    .node
                    .expect("admitted installation retains its Signal node"),
            )
        else {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalContractInstallation,
                "correspondence target node is stale",
            )
            .with_lowering_counters(counters));
        };
        counters.correspondence_batch_preparations += 1;
        let (signal_contract, correspondences) = {
            let bridge = &self.bridge;
            let runtime = self
                .signal_runtime
                .get_mut()
                .unwrap_or_else(PoisonError::into_inner);
            let prepared = crate::correspondence::prepare_registered_correspondence_batch(
                bridge,
                admitted.dependency_extension.registrations(),
                runtime.graph(),
            )
            .map_err(|error| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::CorrespondenceAdmission,
                    format!(
                        "conditional correspondence batch was denied without committing partial allocation: {error:?}"
                    ),
                )
                .with_lowering_counters(counters)
            })?;
            let dependency_aspects = prepared.dependency_aspects();
            let condition_aspects = prepared
                .condition_aspects(admitted.request.contract.condition_dependency_ordinals());
            counters.signal_contract_lowerings += 1;
            let definition = super::lowering::lower_signal_contract(
                &admitted.request.contract,
                dependency_aspects,
                condition_aspects,
            )
            .map_err(|denial| denial.with_lowering_counters(counters))?;
            let signal_contract = runtime
                .graph_mut()
                .install_conditional_contract(
                    &bridge.signal_aspect_lowering_owner,
                    node_capability,
                    definition,
                )
                .map_err(|_| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::SignalContractInstallation,
                        "Signal rejected the owner-bound conditional contract",
                    )
                    .with_lowering_counters(counters)
                })?;
            (signal_contract, prepared.commit())
        };
        self.commit_conditional_lowering(admitted, signal_contract, correspondences, None, counters)
    }

    fn admit_conditional_installation_request(
        &self,
        mut request: BridgeConditionalInstallationRequest,
        owned_node: Option<worth_signal::facade::NodeId>,
    ) -> Result<AdmittedConditionalInstallationRequest, BridgeConditionalDenial> {
        let mut counters = BridgeInstalledConditionalLoweringCounters::default();
        counters.contract_admission_checks += 1;
        super::installation_admission::validate_declaration_pairing(&request)
            .map_err(|denial| denial.with_lowering_counters(counters))?;
        counters.provider_checks += super::provider_admission::PROVIDER_DIMENSION_CHECK_COUNT;
        let provider_admission =
            super::provider_admission::admit_provider_set(&request.contract, &request.providers)
                .map_err(|denial| denial.with_lowering_counters(counters))?;
        let signal_branch_identity = {
            let runtime = self.lock_signal_runtime();
            let selected = runtime.current_branch();
            let basis = runtime
                .observe_signal_branch_basis(selected)
                .map_err(|denial| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::SignalExecution,
                        format!("Signal branch identity admission was denied: {denial:?}"),
                    )
                    .with_lowering_counters(counters)
                })?;
            basis.observation().branch_id().clone()
        };
        let node = match owned_node {
            Some(node) if request.registrations.is_empty() => node,
            Some(_) => {
                return Err(BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                    "owner-allocated source-free node cannot accompany correspondence targets",
                )
                .with_lowering_counters(counters));
            }
            None => {
                self.admit_conditional_signal_target(&mut request.registrations, &mut counters)?
            }
        };
        let dependency_extension =
            self.admit_conditional_dependency_extension(&request.registrations, &mut counters)?;
        counters.semantic_observation_plan_compilations += 1;
        let semantic_observation_plan =
            super::semantic_observation_plan::compile_semantic_observation_plan(
                &request.contract,
                &request.registrations,
            )
            .map_err(|denial| denial.with_lowering_counters(counters))?;
        Ok(AdmittedConditionalInstallationRequest {
            request,
            provider_admission,
            node: Some(node),
            signal_branch_identity,
            dependency_extension,
            semantic_observation_plan,
            counters,
        })
    }

    fn admit_conditional_signal_target(
        &self,
        registrations: &mut [crate::correspondence::BridgeSemanticCorrespondenceRegistration],
        counters: &mut BridgeInstalledConditionalLoweringCounters,
    ) -> Result<worth_signal::facade::NodeId, BridgeConditionalDenial> {
        if registrations.is_empty() {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::EmptyCorrespondenceSet,
                "conditional lowering requires one correspondence per declared dependency",
            )
            .with_lowering_counters(*counters));
        }
        counters.correspondence_registrations_inspected += registrations.len();
        registrations.sort_by_key(|registration| registration.dependency().dependency_ordinal());
        let (graph_instance_id, node) = declared_signal_node(registrations, counters)?;
        counters.signal_graph_checks += 1;
        if graph_instance_id
            != self
                .lock_signal_runtime()
                .graph()
                .installed_graph_capability()
                .graph_instance_id()
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::ForeignSignalGraph,
                "conditional target registrations belong to another Signal graph",
            )
            .with_lowering_counters(*counters));
        }
        counters.signal_node_ownership_checks += 1;
        if self
            .conditional_lowerings
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .any(|lowering| lowering.signal_node() == node)
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalNodeAlreadyBound,
                "a Signal node cannot back multiple conditional declarations before explicit sharing admission",
            )
            .with_lowering_counters(*counters));
        }
        Ok(node)
    }

    fn admit_conditional_dependency_extension(
        &self,
        registrations: &[crate::correspondence::BridgeSemanticCorrespondenceRegistration],
        counters: &mut BridgeInstalledConditionalLoweringCounters,
    ) -> Result<crate::correspondence::AdmittedSemanticDependencyExtension, BridgeConditionalDenial>
    {
        counters.dependency_registry_compilations += 1;
        let extension = self
            .bridge
            .semantic_dependency_registry
            .admit_extension(registrations)
            .map_err(|denial| {
                counters.dependency_registry_existing_key_lookups =
                    denial.counters.existing_key_lookups;
                counters.dependency_registry_batch_key_lookups = denial.counters.batch_key_lookups;
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::CorrespondenceAdmission,
                    format!("{:?}", denial.error),
                )
                .with_lowering_counters(*counters)
            })?;
        counters.dependency_registry_existing_key_lookups =
            extension.counters().existing_key_lookups;
        counters.dependency_registry_batch_key_lookups = extension.counters().batch_key_lookups;
        Ok(extension)
    }
}

pub(super) fn lowering_key(
    lowering: &BridgeInstalledConditionalLowering,
) -> &super::lowering_registry::BridgeConditionalLoweringKey {
    &lowering.registry_key
}

impl Drop for BridgeOwnedSignalRuntime {
    fn drop(&mut self) {
        self.retention.close();
        self.revoke_conditional_liveness();
        self.revoke_managed_clock_liveness();
    }
}

fn declared_signal_node(
    registrations: &[crate::correspondence::BridgeSemanticCorrespondenceRegistration],
    counters: &mut BridgeInstalledConditionalLoweringCounters,
) -> Result<(u64, worth_signal::facade::NodeId), BridgeConditionalDenial> {
    let mut targets = registrations
        .iter()
        .flat_map(|registration| registration.targets.iter());
    let first_target = targets.next().ok_or_else(|| {
        BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::EmptyCorrespondenceSet,
            "conditional registrations retained no Signal target",
        )
        .with_lowering_counters(*counters)
    })?;
    counters.correspondence_targets_inspected += 1;
    let graph = first_target.graph_instance_id();
    let first = first_target.node;
    for target in targets {
        counters.correspondence_targets_inspected += 1;
        if target.graph_instance_id() != graph || target.node != first {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::MixedSignalNodes,
                "one conditional declaration cannot lower across multiple Signal nodes",
            )
            .with_lowering_counters(*counters));
        }
    }
    Ok((graph, first))
}
