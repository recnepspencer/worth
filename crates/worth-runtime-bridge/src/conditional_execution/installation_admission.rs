use super::contract::AdmittedConditionalInstallationRequest;
use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalInstallationRequest,
};
use super::{BridgeInstalledConditionalLoweringCounters, BridgeOwnedSignalRuntime};
use std::sync::{Arc, PoisonError};

pub(super) fn validate_declaration_pairing(
    request: &BridgeConditionalInstallationRequest,
) -> Result<(), BridgeConditionalDenial> {
    if !request.location.is_valid()
        || !request.contract.is_valid()
        || request.location.node_identity() != request.contract.identity()
    {
        return Err(denial(
            BridgeConditionalDenialKind::DeclarationLocationMismatch,
            "conditional location does not name the retained Bridge contract",
        ));
    }
    if request.registrations.len() != request.contract.dependency_count() {
        return Err(denial(
            BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
            "conditional correspondence count differs from the declared dependency count",
        ));
    }
    for ordinal in 0..request.contract.dependency_count() {
        let registration = request
            .registrations
            .iter()
            .find(|item| item.dependency().dependency_ordinal() == ordinal)
            .ok_or_else(|| {
                denial(
                    BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                    "conditional correspondence omitted or duplicated a dependency ordinal",
                )
            })?;
        let dependency = registration.dependency();
        if dependency.source_node_identity() != request.location.node_identity()
            || dependency.source_stage_identity.as_deref() != request.location.stage_identity()
        {
            return Err(denial(
                BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                "conditional correspondence belongs to another source declaration",
            ));
        }
    }
    Ok(())
}

fn denial(kind: BridgeConditionalDenialKind, detail: &'static str) -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(kind, detail)
}

impl BridgeOwnedSignalRuntime {
    pub(super) fn admit_conditional_installation_request(
        &self,
        mut request: BridgeConditionalInstallationRequest,
        owned_node: Option<worth_signal::facade::NodeId>,
    ) -> Result<AdmittedConditionalInstallationRequest, BridgeConditionalDenial> {
        let mut counters = BridgeInstalledConditionalLoweringCounters::default();
        counters.contract_admission_checks += 1;
        super::installation_admission::validate_declaration_pairing(&request)
            .map_err(|denial| denial.with_lowering_counters(counters))?;
        counters.provider_checks += super::provider_admission::PROVIDER_DIMENSION_CHECK_COUNT;
        let provider_admission = super::provider_admission::admit_provider_set(
            Arc::new(request.contract.clone()),
            &request.providers,
        )
        .map_err(|denial| denial.with_lowering_counters(counters))?;
        let signal_branch_identity = {
            let runtime = self.lock_signal_runtime();
            let selected = runtime.current_branch();
            let basis = runtime
                .observe_signal_branch_basis(selected)
                .map_err(|denial| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::SignalExecution(
                            super::BridgeSignalDenial::BranchObservation(denial.clone()),
                        ),
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

    pub(super) fn admit_conditional_signal_target(
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
        let (graph_instance_id, node) =
            super::declared_signal_node::declared_signal_node(registrations, counters)?;
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

    pub(super) fn admit_conditional_dependency_extension(
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
