use worth_signal::facade::{
    InstalledSignalComparatorIdentity, InstalledSignalConditionDecision,
    InstalledSignalConditionIdentity,
};

use super::{
    BridgeConditionalDenial, BridgeConditionalResolverContext, BridgeInstalledConditionalLowering,
};

pub(super) struct ConditionAdapter<'a> {
    lowering: &'a BridgeInstalledConditionalLowering,
    snapshot: Option<
        &'a crate::snapshot::AdmittedSnapshotContext<Box<dyn crate::snapshot::TruthSnapshotReader>>,
    >,
    previous: &'a super::observation_retention::BridgeRetainedObservations,
    managed_source_record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    truth_branch_identity: Option<&'a str>,
    truth_snapshot_identity: &'a str,
    observations: super::observation_retention::BridgeRetainedObservations,
    observation_denial: Option<BridgeConditionalDenial>,
    context_reservation: Option<&'a std::sync::Arc<super::retention::BridgeRetentionReservation>>,
    ledger: &'a std::sync::Arc<super::retention::BridgeRetentionLedger>,
}

impl<'a> ConditionAdapter<'a> {
    pub(super) fn new(
        lowering: &'a BridgeInstalledConditionalLowering,
        snapshot: Option<
            &'a crate::snapshot::AdmittedSnapshotContext<
                Box<dyn crate::snapshot::TruthSnapshotReader>,
            >,
        >,
        previous: &'a super::observation_retention::BridgeRetainedObservations,
        managed_source_record: Option<
            crate::relational_identity::RelationalBridgeRecordIdentityParts,
        >,
        truth_branch_identity: Option<&'a str>,
        truth_snapshot_identity: &'a str,
        ledger: &'a std::sync::Arc<super::retention::BridgeRetentionLedger>,
        context_reservation: Option<
            &'a std::sync::Arc<super::retention::BridgeRetentionReservation>,
        >,
    ) -> Self {
        Self {
            lowering,
            snapshot,
            previous,
            managed_source_record,
            truth_branch_identity,
            truth_snapshot_identity,
            observations: Default::default(),
            ledger,
            context_reservation,
            observation_denial: None,
        }
    }

    pub(super) fn take_observation_denial(&mut self) -> Option<BridgeConditionalDenial> {
        self.observation_denial.take()
    }

    pub(super) fn observation_count(&self) -> usize {
        self.observations.len()
    }

    pub(super) fn take_observations(
        &mut self,
    ) -> super::observation_retention::BridgeRetainedObservations {
        std::mem::take(&mut self.observations)
    }
}

impl worth_signal::facade::InstalledSignalConditionResolver for ConditionAdapter<'_> {
    fn resolve(
        &mut self,
        identity: &InstalledSignalConditionIdentity,
        context: &worth_signal::facade::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, worth_signal::facade::SignalError> {
        if !self
            .lowering
            .signal_contract()
            .accepts_condition_identity(identity)
        {
            return Err(worth_signal::facade::SignalError::invalid_input(
                "installed condition identity did not match its retained Bridge lowering",
            ));
        }
        self.observations = match super::semantic_observations::read_condition_observations(
            self.snapshot,
            self.lowering,
            self.previous,
            self.managed_source_record,
            self.ledger,
        ) {
            Ok(observations) => observations,
            Err(denial) => {
                let detail = denial.detail().to_string();
                self.observation_denial = Some(denial);
                return Err(worth_signal::facade::SignalError::invalid_input(detail));
            }
        };
        if let worth_signal::facade::SignalConditionalCondition::DeltaThreshold(threshold) =
            self.lowering.signal_contract().semantic_condition()
        {
            let observation = self.observations.first().ok_or_else(|| {
                worth_signal::facade::SignalError::invalid_input(
                    "installed semantic threshold retained no admitted observation",
                )
            })?;
            let current = scalar_value(observation.current().ok_or_else(|| {
                worth_signal::facade::SignalError::invalid_input(
                    "installed semantic threshold observed an absent current value",
                )
            })?)?;
            let previous = observation.previous().map(scalar_value).transpose()?;
            return worth_signal::facade::resolve_signal_delta_threshold(
                threshold, previous, current,
            );
        }
        let bridge_context = BridgeConditionalResolverContext::new(
            context.dirty_aspects,
            context.max_dependency_delta,
            self.truth_branch_identity,
            self.truth_snapshot_identity,
            self.observations.clone(),
            std::sync::Arc::clone(
                self.context_reservation
                    .expect("Bridge provider context is preflighted"),
            ),
        );
        if let Some(provider) = &self.lowering.providers.condition {
            return provider
                .resolve(bridge_context)
                .map_err(worth_signal::facade::SignalError::invalid_input);
        }
        if let Some(provider) = &self.lowering.providers.wake {
            return provider
                .resolve(bridge_context)
                .map_err(worth_signal::facade::SignalError::invalid_input);
        }
        Err(worth_signal::facade::SignalError::invalid_input(
            "installed condition lost its exact Bridge provider",
        ))
    }
}

fn scalar_value(
    artifact: &worth_foundational::facade::ContractValidatedAspectArtifact,
) -> Result<&worth_foundational::facade::AspectValue, worth_signal::facade::SignalError> {
    match artifact.payload().view() {
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(value) => Ok(value),
        worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => {
            Err(worth_signal::facade::SignalError::invalid_input(
                "semantic threshold observation was not the admitted scalar projection",
            ))
        }
    }
}

pub(super) struct ComparatorAdapter<'a> {
    lowering: &'a BridgeInstalledConditionalLowering,
}

impl<'a> ComparatorAdapter<'a> {
    pub(super) fn new(lowering: &'a BridgeInstalledConditionalLowering) -> Self {
        Self { lowering }
    }
}

impl worth_signal::facade::VersionComparatorResolver for ComparatorAdapter<'_> {
    fn resolve(
        &mut self,
        _key: &str,
        _aspect: worth_signal::facade::Aspect,
        _cached: u64,
        _current: u64,
    ) -> Result<bool, worth_signal::facade::SignalError> {
        Err(worth_signal::facade::SignalError::invalid_input(
            "portable comparator strings are not installed Bridge authority",
        ))
    }

    fn resolve_installed(
        &mut self,
        identity: &InstalledSignalComparatorIdentity,
        aspect: worth_signal::facade::Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, worth_signal::facade::SignalError> {
        let provider = match self
            .lowering
            .signal_contract()
            .classify_comparator_identity(identity)
        {
            Some(worth_signal::facade::InstalledSignalComparatorUse::DependencyVersion) => {
                self.lowering.providers.dependency_comparator.as_ref()
            }
            Some(worth_signal::facade::InstalledSignalComparatorUse::OutputEquivalence) => {
                self.lowering.providers.output_comparator.as_ref()
            }
            Some(worth_signal::facade::InstalledSignalComparatorUse::ArtifactReuse) => {
                self.lowering.providers.reuse_comparator.as_ref()
            }
            None => None,
        }
        .ok_or_else(|| {
            worth_signal::facade::SignalError::invalid_input(
                "installed comparator identity did not match its retained Bridge role",
            )
        })?;
        provider
            .has_meaningful_change(aspect, cached, current)
            .map_err(worth_signal::facade::SignalError::invalid_input)
    }
}

impl worth_signal::facade::ComparatorPolicyResolver for ComparatorAdapter<'_> {
    fn policy_for_node(
        &self,
        _node: worth_signal::facade::NodeId,
        node_override: Option<&worth_signal::facade::VersionComparatorPolicy>,
    ) -> worth_signal::facade::VersionComparatorPolicy {
        node_override.cloned().unwrap_or_default()
    }
}
