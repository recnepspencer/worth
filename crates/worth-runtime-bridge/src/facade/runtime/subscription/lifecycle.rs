use super::*;

impl RuntimeBridge {
    /// Declares one bridge subscription artifact using the runtime's frozen
    /// Phase 1 declaration-family registry.
    pub fn declare_subscription(
        &self,
        requested_family_kind: BridgeSubscriptionDeclarationFamilyKind,
        normalized_slice_intents: Vec<NormalizedSubscriptionSliceIntent>,
        delivery_intent_class: BridgeSubscriptionDeliveryIntentClass,
    ) -> Result<BridgeSubscriptionDeclaration, BridgeSubscriptionDeclarationRejection> {
        let family_registration = self
            .subscription_family_registry
            .family_for_kind(requested_family_kind);

        BridgeSubscriptionDeclaration::new(
            requested_family_kind,
            delivery_intent_class,
            normalized_slice_intents,
            family_registration,
        )
    }

    /// Returns the frozen subscription-family registry identity bound into this
    /// runtime.
    pub fn subscription_family_registry_identity(
        &self,
    ) -> &BridgeSubscriptionFamilyRegistryIdentity {
        self.subscription_family_registry.registry_identity()
    }

    /// Returns the frozen subscription-family registry counters bound into this
    /// runtime.
    pub fn subscription_family_registry_counters(&self) -> &BridgeSubscriptionCounters {
        self.subscription_family_registry.counters()
    }

    /// Admits one declared bridge subscription against an explicit truth basis
    /// and lowers it to a canonical signal strategy descriptor.
    pub fn admit_subscription(
        &self,
        declaration: &BridgeSubscriptionDeclaration,
        basis_request: BridgeSubscriptionBasisRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<AdmittedBridgeSubscription, BridgeSubscriptionAdmissionRejection> {
        AdmittedBridgeSubscription::admit(self, declaration, basis_request, execution)
    }

    /// Prepares an admitted subscription for activation without performing any
    /// signal registration or delivery fanout.
    pub fn prepare_subscription_activation(
        &self,
        admitted: &AdmittedBridgeSubscription,
    ) -> BridgeSubscriptionActivationReady {
        BridgeSubscriptionActivationReady::prepare(
            self.subscription_family_registry_identity(),
            admitted,
        )
    }

    /// Produces a deactivated retained artifact from one activation-ready
    /// subscription handle.
    pub fn deactivate_subscription(
        &self,
        activation_ready: BridgeSubscriptionActivationReady,
    ) -> BridgeSubscriptionDeactivated {
        let _ = self;
        activation_ready.deactivate()
    }

    /// Builds a retained explanation for one activation-ready subscription.
    pub fn inspect_activation_ready_subscription(
        &self,
        activation_ready: &BridgeSubscriptionActivationReady,
    ) -> BridgeSubscriptionExplanation {
        let _ = self;
        BridgeSubscriptionExplanation::from_activation_ready(activation_ready)
    }

    /// Builds a retained explanation for one deactivated subscription.
    pub fn inspect_deactivated_subscription(
        &self,
        deactivated: &BridgeSubscriptionDeactivated,
    ) -> BridgeSubscriptionExplanation {
        let _ = self;
        BridgeSubscriptionExplanation::from_deactivated(deactivated)
    }

    /// Builds a retained explanation for one subscription admission rejection.
    pub fn inspect_subscription_admission_rejection(
        &self,
        rejection: &BridgeSubscriptionAdmissionRejection,
    ) -> BridgeSubscriptionExplanation {
        let _ = self;
        BridgeSubscriptionExplanation::from_rejection(rejection)
    }

    /// Reconstructs retained subscription meaning from a canonical bundle.
    pub fn replay_subscription(
        &self,
        retained_bundle: &BridgeRetainedSubscriptionBundle,
    ) -> Result<BridgeSubscriptionReplaySummary, BridgeSubscriptionReplayMismatch> {
        BridgeSubscriptionReplaySummary::replay(
            self.subscription_family_registry_identity(),
            retained_bundle,
        )
    }
}
