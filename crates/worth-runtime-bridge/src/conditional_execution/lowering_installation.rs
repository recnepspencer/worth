use std::sync::{Arc, PoisonError};

use super::{
    contract::AdmittedConditionalInstallationRequest, BridgeConditionalDenial,
    BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeInstalledConditionalLoweringCounters, BridgeOwnedSignalRuntime,
};
use crate::correspondence::BridgeInstalledSemanticCorrespondence;

pub(super) struct BridgePreparedConditionalActivation {
    pub(super) predecessor: Arc<BridgeInstalledConditionalLowering>,
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
    slot: super::lowering_registry::BridgeConditionalLoweringSlotClaim,
    targets: super::owned_target_index::BridgeOwnedConditionalTargetReservation,
}

impl BridgeOwnedSignalRuntime {
    pub(super) fn commit_conditional_lowering(
        &mut self,
        admitted: AdmittedConditionalInstallationRequest,
        signal_contract: worth_signal::facade::InstalledSignalConditionalContract,
        correspondences: Vec<BridgeInstalledSemanticCorrespondence>,
        definition_custody: Option<
            worth_signal::facade::branch::SignalConditionalInstallationCustody,
        >,
        mut counters: BridgeInstalledConditionalLoweringCounters,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        counters.dependency_registry_commits = admitted
            .dependency_extension
            .commit(&mut self.bridge.semantic_dependency_registry);
        counters.correspondence_admissions = correspondences.len();
        counters.signal_targets_joined = correspondences
            .iter()
            .map(BridgeInstalledSemanticCorrespondence::target_count)
            .sum();
        counters.signal_contract_installations = 1;
        let registry_key = super::lowering_registry::BridgeConditionalLoweringKey::new(
            admitted.signal_branch_identity.clone(),
            &signal_contract,
        );
        let lowering = self.build_installed_lowering(
            registry_key,
            admitted.request.contract,
            admitted.request.location,
            Arc::from(correspondences),
            admitted.semantic_observation_plan,
            signal_contract,
            definition_custody,
            admitted.request.providers,
            admitted.provider_admission,
            counters,
        );
        self.owned_conditional_targets
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .register(&lowering);
        self.conditional_lowerings
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .install(
                super::contract::lowering_key(&lowering).clone(),
                Arc::clone(&lowering),
            );
        Ok(lowering)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_conditional_definition_activation(
        &self,
        predecessor: Arc<BridgeInstalledConditionalLowering>,
        signal_branch_identity: worth_signal::facade::branch::SignalBranchIdentity,
        contract: super::BridgeConditionalContract,
        location: super::BridgeConditionalLocation,
        providers: super::BridgeConditionalProviderSet,
        provider_admission: super::provider_admission::BridgeConditionalProviderAdmission,
        semantic_observation_plan: Option<
            super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan,
        >,
        mut counters: BridgeInstalledConditionalLoweringCounters,
    ) -> Result<BridgePreparedConditionalActivation, BridgeConditionalDenial> {
        let expected_generation = predecessor
            .signal_definition_generation()
            .checked_add(1)
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalContractInstallation,
                    "conditional definition generation is exhausted",
                )
            })?;
        counters.signal_node_admissions = 1;
        counters.signal_contract_installations = 1;
        let correspondences = Arc::clone(&predecessor.correspondences);
        let projection_basis = super::lowering_identity::installed_lowering_identity(
            self.bridge.signal_runtime_key,
            &signal_branch_identity,
            predecessor.signal_contract(),
            contract.identity(),
            &correspondences,
        );
        let retained_bytes = super::retention::definition_candidate_retained_bytes(
            &signal_branch_identity,
            &contract,
            &location,
            &providers,
            projection_basis.len(),
            correspondences.len(),
            semantic_observation_plan.as_ref(),
        )
        .map_err(super::observation_retention::retention_denial)?;
        let definition_retention = Arc::new(
            self.retention
                .reserve_definition_candidate(retained_bytes)
                .map_err(super::observation_retention::retention_denial)?,
        );
        let key = super::lowering_registry::BridgeConditionalLoweringKey::successor(
            signal_branch_identity,
            predecessor.signal_node(),
            expected_generation,
        );
        let slot = super::lowering_registry::BridgeConditionalLoweringRegistry::claim(
            &self.conditional_lowerings,
            key.clone(),
        )?;
        let targets =
            super::owned_target_index::BridgeOwnedConditionalTargetIndex::reserve_existing(
                &self.owned_conditional_targets,
                Arc::clone(&correspondences),
            )?;
        let (authority, projection) =
            super::lowering_authority::mint_bridge_conditional_lowering_identity(projection_basis);
        let lowering = Arc::new(BridgeInstalledConditionalLowering {
            bridge_runtime_key: self.bridge.signal_runtime_key,
            registry_key: key,
            _authority: authority,
            projection,
            contract,
            location,
            correspondences: Arc::clone(&correspondences),
            semantic_observation_plan,
            signal_contract: Default::default(),
            signal_port: Default::default(),
            definition_custody: Default::default(),
            providers,
            provider_admission,
            lease: Arc::new(super::liveness::BridgeConditionalLoweringLease::issue()),
            _definition_retention: Some(definition_retention),
            counters,
        });
        lowering.reserve_signal_port();
        Ok(BridgePreparedConditionalActivation {
            predecessor,
            lowering,
            slot,
            targets,
        })
    }

    pub(super) fn admit_conditional_activation_contract(
        &self,
        activation: &BridgePreparedConditionalActivation,
        signal_contract: &worth_signal::facade::InstalledSignalConditionalContract,
    ) -> Result<(), BridgeConditionalDenial> {
        let expected_generation = activation
            .predecessor
            .signal_definition_generation()
            .checked_add(1)
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalContractInstallation,
                    "conditional definition generation is exhausted",
                )
            })?;
        if signal_contract.node() != activation.predecessor.signal_node()
            || signal_contract.graph_instance_id()
                != activation.predecessor.signal_graph_instance_id()
            || signal_contract.generation() != expected_generation
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "Signal completion did not advance the exact installed Bridge lowering",
            ));
        }
        self.require_live_installed_lowering(&activation.predecessor)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_installed_lowering(
        &self,
        registry_key: super::lowering_registry::BridgeConditionalLoweringKey,
        contract: super::BridgeConditionalContract,
        location: super::BridgeConditionalLocation,
        correspondences: Arc<[BridgeInstalledSemanticCorrespondence]>,
        semantic_observation_plan: Option<
            super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan,
        >,
        signal_contract: worth_signal::facade::InstalledSignalConditionalContract,
        definition_custody: Option<
            worth_signal::facade::branch::SignalConditionalInstallationCustody,
        >,
        providers: super::BridgeConditionalProviderSet,
        provider_admission: super::provider_admission::BridgeConditionalProviderAdmission,
        counters: BridgeInstalledConditionalLoweringCounters,
    ) -> Arc<BridgeInstalledConditionalLowering> {
        let (authority, projection) =
            super::lowering_authority::mint_bridge_conditional_lowering_identity(
                super::lowering_identity::installed_lowering_identity(
                    self.bridge.signal_runtime_key,
                    registry_key.branch(),
                    &signal_contract,
                    contract.identity(),
                    &correspondences,
                ),
            );
        let lowering = Arc::new(BridgeInstalledConditionalLowering {
            bridge_runtime_key: self.bridge.signal_runtime_key,
            registry_key,
            _authority: authority,
            projection,
            contract,
            location,
            correspondences,
            semantic_observation_plan,
            signal_contract: std::sync::OnceLock::from(signal_contract),
            signal_port: Default::default(),
            definition_custody: Default::default(),
            providers,
            provider_admission,
            lease: Arc::new(super::liveness::BridgeConditionalLoweringLease::issue()),
            _definition_retention: None,
            counters,
        });
        if let Some(custody) = definition_custody {
            lowering.initialize_definition_custody(custody);
        }
        lowering
    }

    pub(super) fn commit_admitted_conditional_activation(
        &self,
        admitted: BridgePreparedConditionalActivation,
    ) -> Arc<BridgeInstalledConditionalLowering> {
        let BridgePreparedConditionalActivation {
            lowering,
            slot,
            targets,
            ..
        } = admitted;
        targets.commit();
        slot.publish(Arc::clone(&lowering));
        if let Some(services) = self.signal_services.as_ref() {
            services.publish_conditional_port_from_lowering(&lowering);
        }
        lowering
    }
}
