use super::*;
use crate::temporal::{
    BridgeTemporalBasisDenial, BridgeTemporalSignalBasis, BridgeTemporalSignalBasisDenial,
    BridgeTemporalTruthViewBasis, BridgeTemporalWakeEvidence,
};
use worth_proof::TransitionOutcome;
use worth_signal::facade::{
    branch::SignalConditionalTemporalPromotion, ClockDomain, TemporalPreviousValueReference,
    TemporalWakeId,
};

impl RuntimeBridge {
    /// Admits a temporal basis for a wake retained in a Signal promotion result.
    ///
    /// ```compile_fail,E0308
    /// use worth_runtime_bridge::facade::{
    ///     BridgeTemporalSignalBasis, BridgeTemporalTruthViewBasis, RuntimeBridge,
    /// };
    /// use worth_signal::facade::TemporalWakeId;
    /// fn forge(runtime: &RuntimeBridge, truth: BridgeTemporalTruthViewBasis,
    ///          signal: BridgeTemporalSignalBasis) {
    ///     let _ = runtime.admit_promoted_temporal_basis(truth, signal, &"wake", TemporalWakeId::new(1));
    /// }
    /// ```
    pub fn admit_promoted_temporal_basis(
        &self,
        truth_basis: BridgeTemporalTruthViewBasis,
        signal_basis: BridgeTemporalSignalBasis,
        promotion: &SignalConditionalTemporalPromotion,
        wake_id: TemporalWakeId,
    ) -> TransitionOutcome<AdmittedBridgeTemporalBasis, BridgeTemporalBasisDenial> {
        let promotion = promotion.promotion();
        let Some(wake) = promotion
            .ready_wakes()
            .iter()
            .find(|wake| wake.id() == wake_id)
        else {
            return TransitionOutcome::denied(BridgeTemporalBasisDenial::MissingWakeEvidence);
        };
        if wake.clock_domain() != ClockDomain::MonotonicExecution {
            return TransitionOutcome::denied(BridgeTemporalBasisDenial::SignalBasisDenied(
                BridgeTemporalSignalBasisDenial::WrongClockDomain {
                    domain: wake.clock_domain(),
                },
            ));
        }
        AdmittedBridgeTemporalBasis::admit(
            truth_basis,
            signal_basis,
            Some(BridgeTemporalWakeEvidence::new(
                wake.id(),
                wake.ready_ordinal(),
                wake.ready_tick(),
            )),
        )
    }

    /// Admits one time-aware subscription family over an already-admitted
    /// bridge subscription and a sealed temporal bridge basis artifact.
    pub fn admit_temporal_subscription(
        &self,
        admitted: &AdmittedBridgeSubscription,
        temporal_basis: AdmittedBridgeTemporalBasis,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
    ) -> Result<AdmittedTemporalBridgeSubscription, BridgeTemporalSubscriptionAdmissionRejection>
    {
        let _ = self;
        AdmittedTemporalBridgeSubscription::admit(admitted, temporal_basis, family_kind)
    }

    /// Prepares a time-aware subscription for activation without collapsing the
    /// temporal admission proof into the ordinary activation-ready handle.
    pub fn prepare_temporal_subscription_activation(
        &self,
        temporal_admission: &AdmittedTemporalBridgeSubscription,
    ) -> BridgeTemporalSubscriptionActivationReady {
        BridgeTemporalSubscriptionActivationReady::prepare(
            self.subscription_family_registry_identity(),
            temporal_admission,
        )
    }

    /// Admits one preview-scoped time-aware subscription family over an
    /// already-admitted bridge subscription, a sealed preview basis artifact,
    /// and a sealed temporal bridge basis artifact.
    pub fn admit_preview_temporal_subscription(
        &self,
        admitted: &AdmittedBridgeSubscription,
        preview_basis: &BridgeSubscriptionPreviewBasisBinding,
        temporal_basis: AdmittedBridgeTemporalBasis,
        family_kind: BridgeTemporalSubscriptionFamilyKind,
    ) -> Result<
        AdmittedPreviewTemporalBridgeSubscription,
        BridgePreviewTemporalSubscriptionAdmissionRejection,
    > {
        let _ = self;
        AdmittedPreviewTemporalBridgeSubscription::admit(
            admitted,
            preview_basis,
            temporal_basis,
            family_kind,
        )
    }

    /// Prepares a preview-scoped time-aware subscription for activation
    /// without collapsing its preview or temporal proof into the ordinary
    /// activation-ready handle.
    pub fn prepare_preview_temporal_subscription_activation(
        &self,
        preview_temporal_admission: &AdmittedPreviewTemporalBridgeSubscription,
    ) -> BridgePreviewTemporalSubscriptionActivationReady {
        BridgePreviewTemporalSubscriptionActivationReady::prepare(
            self.subscription_family_registry_identity(),
            preview_temporal_admission,
        )
    }

    /// Freezes one authoritative temporal wake routing request as a distinct
    /// proof step between activation readiness and routed cause construction.
    pub fn prepare_temporal_wake_routing(
        &self,
        activation_ready: &BridgeTemporalSubscriptionActivationReady,
    ) -> BridgeTemporalWakeRoutingRequest {
        let _ = self;
        BridgeTemporalWakeRoutingRequest::prepare_authoritative(activation_ready)
    }

    /// Freezes one preview-scoped temporal wake routing request as a distinct
    /// proof step between activation readiness and routed cause construction.
    pub fn prepare_preview_temporal_wake_routing(
        &self,
        activation_ready: &BridgePreviewTemporalSubscriptionActivationReady,
    ) -> BridgeTemporalWakeRoutingRequest {
        let _ = self;
        BridgeTemporalWakeRoutingRequest::prepare_preview(activation_ready)
    }

    /// Routes one promoted temporal wake through the bridge without implying a
    /// new relational patch cause.
    pub fn route_temporal_wake(
        &self,
        request: &BridgeTemporalWakeRoutingRequest,
        prior_cause: Option<&BridgeTemporalCauseRecord>,
    ) -> Result<BridgeTemporalCauseRecord, BridgeTemporalWakeRoutingRejection> {
        let _ = self;
        BridgeTemporalCauseRecord::route_wake(request, prior_cause)
    }

    /// Routes one temporal wake together with one explicit committed patch
    /// while preserving both cause identities in the routed artifact.
    pub fn route_temporal_wake_with_truth_patch(
        &self,
        request: &BridgeTemporalWakeRoutingRequest,
        truth_patch: &BridgeCommittedPatchEnvelope,
        prior_cause: Option<&BridgeTemporalCauseRecord>,
    ) -> Result<BridgeTemporalCauseRecord, BridgeTemporalWakeRoutingRejection> {
        let _ = self;
        BridgeTemporalCauseRecord::route_wake_with_truth_patch(request, truth_patch, prior_cause)
    }

    /// Plans one temporal delivery window descriptor from a routed temporal
    /// cause without yet opening a canonical active-subscription window.
    pub fn plan_temporal_delivery_window(
        &self,
        cause_record: &BridgeTemporalCauseRecord,
        delivery_family_kind: BridgeSubscriptionDeliveryFamilyKind,
    ) -> BridgeTemporalDeliveryWindowPlan {
        let _ = self;
        cause_record.plan_delivery_window(delivery_family_kind)
    }

    /// Freezes one historical-only truth view basis from an already-admitted
    /// temporal truth basis without reopening current-truth basis selection.
    pub fn admit_historical_truth_view_basis(
        &self,
        truth_basis: &crate::facade::AdmittedBridgeTemporalTruthViewBasis,
    ) -> Result<AdmittedBridgeHistoricalTruthViewBasis, BridgeHistoricalTruthBasisAdmissionRejection>
    {
        let _ = self;
        AdmittedBridgeHistoricalTruthViewBasis::admit(truth_basis)
    }

    /// Seals retained previous-value evidence for historical replay against one
    /// explicit pinned truth snapshot.
    pub fn retain_historical_previous_value_evidence(
        &self,
        truth_branch_identity: TruthBranchIdentity,
        truth_snapshot_identity: TruthSnapshotIdentity,
        references: Vec<TemporalPreviousValueReference>,
    ) -> RetainedHistoricalPreviousValueEvidence {
        let _ = self;
        RetainedHistoricalPreviousValueEvidence::retain(
            truth_branch_identity,
            truth_snapshot_identity,
            references,
        )
    }

    /// Admits one historical temporal replay basis over an already-admitted
    /// historical temporal subscription and explicit retained evidence.
    pub fn admit_historical_temporal_replay_basis(
        &self,
        temporal_admission: &AdmittedTemporalBridgeSubscription,
        historical_truth_basis: &AdmittedBridgeHistoricalTruthViewBasis,
        retained_previous_values: RetainedHistoricalPreviousValueEvidence,
    ) -> Result<AdmittedHistoricalTemporalReplayBasis, BridgeHistoricalTemporalReplayRejection>
    {
        let _ = self;
        AdmittedHistoricalTemporalReplayBasis::admit(
            temporal_admission,
            historical_truth_basis,
            retained_previous_values,
        )
    }

    /// Freezes one historical replay request as a distinct proof step between
    /// retained replay basis admission and historical temporal readiness.
    pub fn prepare_historical_temporal_replay_request(
        &self,
        replay_basis: &AdmittedHistoricalTemporalReplayBasis,
    ) -> BridgeHistoricalTemporalSubscriptionReplayRequest {
        let _ = self;
        BridgeHistoricalTemporalSubscriptionReplayRequest::prepare(replay_basis)
    }

    /// Prepares one historical temporal readiness artifact strictly from the
    /// retained historical replay proof chain.
    pub fn prepare_historical_temporal_readiness(
        &self,
        replay_request: &BridgeHistoricalTemporalSubscriptionReplayRequest,
    ) -> BridgeHistoricalTemporalReadiness {
        BridgeHistoricalTemporalReadiness::prepare(
            self.subscription_family_registry_identity(),
            replay_request,
        )
    }
}
