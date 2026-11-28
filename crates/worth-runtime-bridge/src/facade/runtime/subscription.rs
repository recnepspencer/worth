mod certification;
mod certification_closeout;
mod delivery_admission;
mod lifecycle;
mod mixed_cause;
mod preview_lifecycle;
mod resume_basis;
mod shared_delivery;
mod temporal;

use super::*;

impl RuntimeBridge {
    /// Opens a phase-typed delivery window for one active subscription.
    pub fn open_subscription_delivery_window(
        &self,
        active_subscription: &BridgeActiveSubscription,
        delivery_family_kind: BridgeSubscriptionDeliveryFamilyKind,
        delivery_window_sequence: u64,
    ) -> BridgeSubscriptionDeliveryWindowOpen {
        let _ = self;
        BridgeSubscriptionDeliveryWindowOpen::open(
            active_subscription,
            delivery_family_kind,
            delivery_window_sequence,
        )
    }

    /// Seals a delivery window into canonical delivery member records.
    pub fn seal_subscription_delivery_window(
        &self,
        delivery_window: BridgeSubscriptionDeliveryWindowOpen,
        members: Vec<BridgeSubscriptionDeliveryMemberInput>,
    ) -> Result<BridgeSubscriptionDeliveryWindowSealed, BridgeSubscriptionDeliveryWindowRejection>
    {
        let _ = self;
        delivery_window.seal(members)
    }

    /// Returns the hot-path diagnostics reference for a sealed delivery window.
    pub fn inspect_subscription_delivery_reference<'a>(
        &self,
        sealed: &'a BridgeSubscriptionDeliveryWindowSealed,
    ) -> &'a BridgeSubscriptionDeliveryDiagnosticsReference {
        let _ = self;
        sealed.diagnostics_reference()
    }

    /// Admits additional equivalent consumers to share one active subscription.
    pub fn plan_shared_subscription_fanout(
        &self,
        active_subscription: &BridgeActiveSubscription,
        additional_consumers: Vec<BridgeSubscriptionConsumerContract>,
    ) -> Result<BridgeSubscriptionFanoutPlan, BridgeSubscriptionFanoutPlanRejection> {
        let _ = self;
        BridgeSubscriptionFanoutPlan::plan(active_subscription, additional_consumers)
    }

    /// Lowers an admitted fanout plan into a compact indexed layout before
    /// delivery projection.
    pub fn build_subscription_fanout_layout(
        &self,
        fanout_plan: BridgeSubscriptionFanoutPlan,
        delivery_family_kind: BridgeSubscriptionDeliveryFamilyKind,
    ) -> BridgeSubscriptionFanoutLayout {
        let _ = self;
        BridgeSubscriptionFanoutLayout::build(fanout_plan, delivery_family_kind)
    }

    /// Projects one sealed canonical delivery window through a prebuilt fanout
    /// layout without cloning canonical member records per consumer.
    pub fn project_subscription_delivery_to_fanout(
        &self,
        fanout_layout: &BridgeSubscriptionFanoutLayout,
        sealed_window: &BridgeSubscriptionDeliveryWindowSealed,
    ) -> Result<
        BridgeSubscriptionFanoutDeliveryProjectionSet,
        BridgeSubscriptionFanoutProjectionRejection,
    > {
        let _ = self;
        BridgeSubscriptionFanoutDeliveryProjection::project(fanout_layout, sealed_window)
    }

    /// Retains descriptor evidence for a sealed delivery window without
    /// reconstructing delivered content or rich diagnostics.
    pub fn retain_subscription_delivery_window_seed(
        &self,
        sealed_window: &BridgeSubscriptionDeliveryWindowSealed,
    ) -> BridgeSubscriptionRetainedDeliveryWindowSeed {
        let _ = self;
        BridgeSubscriptionRetainedDeliveryWindowSeed::retain(sealed_window)
    }

    /// Retains descriptor evidence for a fanout projection set without replaying
    /// delivery.
    pub fn retain_subscription_fanout_projection_seed(
        &self,
        projection_set: &BridgeSubscriptionFanoutDeliveryProjectionSet,
    ) -> BridgeSubscriptionRetainedDeliveryReplaySeed {
        let _ = self;
        BridgeSubscriptionRetainedDeliveryReplaySeed::retain(projection_set)
    }

    /// Validates a fanout projection set against its layout as a compact proof.
    pub fn validate_subscription_fanout_projection(
        &self,
        fanout_layout: &BridgeSubscriptionFanoutLayout,
        projection_set: &BridgeSubscriptionFanoutDeliveryProjectionSet,
    ) -> Result<
        BridgeSubscriptionFanoutProjectionValidation,
        BridgeSubscriptionFanoutProjectionValidationRejection,
    > {
        let _ = self;
        BridgeSubscriptionFanoutProjectionValidation::validate(fanout_layout, projection_set)
    }

    /// Inspects replay readiness without executing retained replay.
    pub fn inspect_subscription_delivery_replay_readiness(
        &self,
        sealed_window: &BridgeSubscriptionDeliveryWindowSealed,
    ) -> BridgeSubscriptionDeliveryWindowReplayReadiness {
        let _ = self;
        BridgeSubscriptionDeliveryWindowReplayReadiness::inspect(sealed_window)
    }

    /// Plans retained delivery replay from checkpoint-approved resume evidence.
    /// This only admits ordered retained window descriptors; it does not
    /// dispatch callbacks, replay delivered content, or materialize rich diagnostics.
    pub fn plan_subscription_delivery_replay(
        &self,
        active_subscription: &BridgeActiveSubscription,
        resume_admission: BridgeSubscriptionResumeAdmission,
        retained_window_seeds: Vec<BridgeSubscriptionRetainedDeliveryWindowSeed>,
    ) -> Result<BridgeSubscriptionDeliveryReplayPlan, BridgeSubscriptionDeliveryReplayPlanRejection>
    {
        let _ = self;
        BridgeSubscriptionDeliveryReplayPlan::plan(
            active_subscription,
            resume_admission,
            retained_window_seeds,
        )
    }

    /// Admits a preview-scoped subscription basis from an active preview
    /// session and its matching retained preview execution record. Ordinary
    /// branch-head or snapshot basis artifacts cannot satisfy this API.
    pub fn admit_subscription_preview_basis(
        &self,
        active_preview_session: &BridgePreviewSession<PreviewActive>,
        preview_execution_record: &BridgePreviewExecutionRecord,
    ) -> Result<BridgeSubscriptionPreviewBasisBinding, BridgeSubscriptionPreviewBasisRejection>
    {
        let _ = self;
        BridgeSubscriptionPreviewBasisBinding::admit(
            active_preview_session,
            preview_execution_record,
        )
    }

    /// Activates a preview-scoped subscription delivery proof. The returned
    /// type is intentionally distinct from authoritative active subscriptions.
    pub fn activate_preview_subscription_delivery(
        &self,
        activation_ready: BridgeSubscriptionActivationReady,
        preview_basis: BridgeSubscriptionPreviewBasisBinding,
        cost_profile: BridgeSubscriptionDeliveryCostProfile,
        consumer_contract: BridgeSubscriptionConsumerContract,
    ) -> BridgePreviewActiveSubscription {
        let _ = self;
        BridgePreviewActiveSubscription::activate(
            activation_ready,
            preview_basis,
            cost_profile,
            consumer_contract,
        )
    }

    /// Builds a preview-residue scope index from explicit scope-local artifact
    /// descriptors. This does not scan authoritative or global registries.
    pub fn build_subscription_preview_residue_scope_index(
        &self,
        preview_active: &BridgePreviewActiveSubscription,
        artifact_inputs: Vec<BridgeSubscriptionPreviewResidueArtifactInput>,
    ) -> BridgeSubscriptionPreviewResidueScopeIndex {
        let _ = self;
        BridgeSubscriptionPreviewResidueScopeIndex::build(preview_active, artifact_inputs)
    }

    /// Records the preview-scoped routing, delivery, diagnostics, and
    /// continuation work descriptors that justify a later residue proof. This
    /// is intentionally separate from authoritative delivery windows.
    pub fn record_preview_subscription_work(
        &self,
        preview_active: &BridgePreviewActiveSubscription,
        inputs: Vec<BridgeSubscriptionPreviewWorkInput>,
    ) -> Result<BridgeSubscriptionPreviewWorkTrace, BridgeSubscriptionPreviewWorkTraceRejection>
    {
        let _ = self;
        BridgeSubscriptionPreviewWorkTrace::record(preview_active, inputs)
    }

    /// Admits an acknowledged canonical frontier from a sealed delivery window.
    pub fn admit_subscription_acknowledgement_frontier(
        &self,
        sealed_window: &BridgeSubscriptionDeliveryWindowSealed,
        acknowledged_sequence: usize,
        acknowledged_member: &BridgeSubscriptionDeliveryMemberRecord,
    ) -> Result<
        BridgeSubscriptionAcknowledgementFrontier,
        BridgeSubscriptionAcknowledgementFrontierRejection,
    > {
        let _ = self;
        BridgeSubscriptionAcknowledgementFrontier::admit(
            sealed_window,
            acknowledged_sequence,
            acknowledged_member,
        )
    }

    /// Converts an admitted acknowledgement frontier into checkpoint-ready
    /// evidence without publishing a replay token yet.
    pub fn prepare_subscription_checkpoint(
        &self,
        frontier: BridgeSubscriptionAcknowledgementFrontier,
    ) -> BridgeSubscriptionCheckpointReady {
        let _ = self;
        BridgeSubscriptionCheckpointReady::prepare(frontier)
    }

    /// Publishes a checkpoint from checkpoint-ready evidence and the matching
    /// active subscription proof.
    pub fn publish_subscription_checkpoint(
        &self,
        checkpoint_ready: BridgeSubscriptionCheckpointReady,
        active_subscription: &BridgeActiveSubscription,
        duplicate_replay_policy_kind: BridgeSubscriptionDuplicateReplayPolicyKind,
    ) -> Result<
        BridgeSubscriptionCheckpoint,
        (
            BridgeSubscriptionCheckpointReady,
            BridgeSubscriptionCheckpointRejection,
        ),
    > {
        let _ = self;
        BridgeSubscriptionCheckpoint::publish(
            checkpoint_ready,
            active_subscription,
            duplicate_replay_policy_kind,
            None,
        )
    }

    /// Publishes a checkpoint from checkpoint-ready evidence while binding the
    /// compact fanout layout that delivered the acknowledged window.
    pub fn publish_subscription_fanout_checkpoint(
        &self,
        checkpoint_ready: BridgeSubscriptionCheckpointReady,
        active_subscription: &BridgeActiveSubscription,
        fanout_layout: &BridgeSubscriptionFanoutLayout,
        duplicate_replay_policy_kind: BridgeSubscriptionDuplicateReplayPolicyKind,
    ) -> Result<
        BridgeSubscriptionCheckpoint,
        (
            BridgeSubscriptionCheckpointReady,
            BridgeSubscriptionCheckpointRejection,
        ),
    > {
        let _ = self;
        BridgeSubscriptionCheckpoint::publish(
            checkpoint_ready,
            active_subscription,
            duplicate_replay_policy_kind,
            Some(fanout_layout),
        )
    }

    /// Admits a checkpoint for resume against the matching active subscription
    /// without reopening delivery or reconstructing retained delivery content.
    pub fn admit_subscription_resume(
        &self,
        active_subscription: &BridgeActiveSubscription,
        checkpoint: &BridgeSubscriptionCheckpoint,
    ) -> Result<BridgeSubscriptionResumeAdmission, BridgeSubscriptionResumeAdmissionRejection> {
        let _ = self;
        BridgeSubscriptionResumeAdmission::admit(active_subscription, checkpoint)
    }

    /// Lowers resume admission into a descriptor plan for the next canonical
    /// member sequence. Execution remains a later batch.
    pub fn plan_subscription_resume(
        &self,
        resume_admission: BridgeSubscriptionResumeAdmission,
    ) -> BridgeSubscriptionResumePlan {
        let _ = self;
        BridgeSubscriptionResumePlan::plan(resume_admission)
    }

    /// Builds a locality-scoped continuation index from explicit truth-owned
    /// candidate descriptors. This does not scan active subscription registries.
    pub fn build_subscription_continuation_index(
        &self,
        active_subscription: &BridgeActiveSubscription,
        candidate_inputs: Vec<BridgeSubscriptionContinuationCandidateInput>,
    ) -> Result<BridgeSubscriptionContinuationIndex, BridgeSubscriptionContinuationIndexRejection>
    {
        let _ = self;
        BridgeSubscriptionContinuationIndex::build(active_subscription, candidate_inputs)
    }

    /// Plans one typed subscription continuation from a prebuilt locality index.
    pub fn plan_subscription_continuation(
        &self,
        active_subscription: &BridgeActiveSubscription,
        continuation_index: &BridgeSubscriptionContinuationIndex,
        candidate_slot: usize,
    ) -> Result<BridgeSubscriptionContinuationDecision, BridgeSubscriptionContinuationRejection>
    {
        let _ = self;
        BridgeSubscriptionContinuationDecision::plan(
            active_subscription,
            continuation_index,
            candidate_slot,
        )
    }
}
