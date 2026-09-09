use std::sync::Arc;

use super::super::{
    provider_admission, semantic_observation_plan, BridgeConditionalDenial,
    BridgeConditionalDenialKind, BridgeInstalledConditionalLoweringCounters,
    BridgeOwnedConditionalInstallationRequest, BridgeOwnedSignalRuntime,
};

pub struct BridgePreparedConditionalInstallationExtension {
    pub(super) signal_port: super::super::signal_port::BridgeConditionalSignalPort,
    pub(super) signal: worth_signal::facade::branch::SignalConditionalInstallationExtensionRequest,
    pub(super) signal_publication:
        Option<worth_signal::facade::branch::SignalConditionalDefinitionPublicationOperation>,
    pub(super) activation: super::super::lowering_installation::BridgePreparedConditionalActivation,
}

impl BridgeOwnedSignalRuntime {
    pub(crate) fn prepare_owned_conditional_definition_successor(
        &self,
        predecessor: &super::super::BridgeConditionalSignalBasisBinding,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<BridgePreparedConditionalInstallationExtension, BridgeConditionalDenial> {
        let installed = &predecessor.lowering;
        self.require_live_installed_lowering(installed)?;
        super::super::owned_installation::validate_dependency_shape(&request)?;
        if request.contract.identity() != installed.contract().identity()
            || request.location != *installed.location()
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                "a definition successor must retain its installed operation identity and location",
            ));
        }
        if request.dependencies.len() != installed.correspondence_count()
            || !request
                .dependencies
                .iter()
                .zip(installed.semantic_dependencies())
                .all(|(candidate, installed)| candidate == installed)
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::CorrespondenceAdmission,
                "a definition successor must retain every exact installed semantic correspondence",
            ));
        }
        let provider_admission =
            provider_admission::admit_provider_set(&request.contract, &request.providers)?;
        let semantic_observation_plan =
            semantic_observation_plan::compile_semantic_observation_plan_from_dependencies(
                &request.contract,
                request.dependencies.iter(),
            )?;
        let definition = super::super::lowering::lower_signal_contract(
            &request.contract,
            installed.signal_dependency_aspects(),
            installed.signal_condition_aspects(request.contract.condition_dependency_ordinals()),
        )?;
        let counters = BridgeInstalledConditionalLoweringCounters {
            contract_admission_checks: 1,
            correspondence_registrations_inspected: request.dependencies.len(),
            correspondence_targets_inspected: installed
                .correspondences
                .iter()
                .map(crate::correspondence::BridgeInstalledSemanticCorrespondence::target_count)
                .sum(),
            signal_graph_checks: 1,
            signal_node_ownership_checks: 1,
            semantic_observation_plan_compilations: 1,
            signal_contract_lowerings: 1,
            provider_checks: super::super::provider_admission::PROVIDER_DIMENSION_CHECK_COUNT,
            ..Default::default()
        };
        let activation = self.prepare_conditional_definition_activation(
            Arc::clone(installed),
            predecessor
                .signal_port
                .issuance_basis()
                .observation()
                .branch_id()
                .clone(),
            request.contract,
            request.location,
            request.providers,
            provider_admission,
            semantic_observation_plan,
            counters,
        )?;
        let signal_port = predecessor.signal_port.clone();
        let signal = signal_port
            .prepare_installation_extension(
                installed.signal_node(),
                installed.signal_definition_generation(),
                definition,
            )
            .map_err(|denial| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalContractInstallation,
                    format!("Signal successor preparation was denied: {denial:?}"),
                )
            })?;
        let (signal_publication, signal) = signal.into_parts();
        Ok(BridgePreparedConditionalInstallationExtension {
            signal_port,
            signal,
            signal_publication: Some(signal_publication),
            activation,
        })
    }
}

impl BridgePreparedConditionalInstallationExtension {
    #[doc(hidden)]
    pub fn take_runtime_world_publication_operation(
        &mut self,
    ) -> worth_signal::facade::branch::SignalConditionalDefinitionPublicationOperation {
        self.signal_publication
            .take()
            .expect("a prepared Bridge installation has one Runtime World publication operation")
    }
}
