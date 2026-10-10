use std::sync::Arc;

use super::super::WorthQueryProductSharedRoot;

impl WorthQueryProductSharedRoot {
    /// Delivers a performed change only inside this root's installed advancement.
    ///
    /// ```compile_fail,E0308
    /// # use std::sync::Arc;
    /// # use worth_query_execution::facade::{
    /// #     application_contribution::WorthQueryAdvancementPhase,
    /// #     integration::WorthQueryProductSharedRoot,
    /// #     product::WorthQueryPerformedRelationalProductChange,
    /// #     provider_session::ExecutionRequest,
    /// # };
    /// # use worth_runtime_bridge::facade::BridgeInstalledConditionalLowering;
    /// # fn contract(
    /// #     root: &WorthQueryProductSharedRoot,
    /// #     phase: &WorthQueryAdvancementPhase<'_>,
    /// #     request: ExecutionRequest<'_, '_>,
    /// #     lowering: &Arc<BridgeInstalledConditionalLowering>,
    /// #     change: WorthQueryPerformedRelationalProductChange,
    /// # ) {
    /// let _ = root.deliver_performed_relational_change(request, lowering, 0, change);
    /// # }
    /// ```
    /// ```
    /// # use std::sync::Arc;
    /// # use worth_query_execution::facade::{
    /// #     application_contribution::WorthQueryAdvancementPhase,
    /// #     integration::WorthQueryProductSharedRoot,
    /// #     product::WorthQueryPerformedRelationalProductChange,
    /// #     provider_session::ExecutionRequest,
    /// # };
    /// # use worth_runtime_bridge::facade::BridgeInstalledConditionalLowering;
    /// # fn contract(
    /// #     root: &WorthQueryProductSharedRoot,
    /// #     phase: &WorthQueryAdvancementPhase<'_>,
    /// #     request: ExecutionRequest<'_, '_>,
    /// #     lowering: &Arc<BridgeInstalledConditionalLowering>,
    /// #     change: WorthQueryPerformedRelationalProductChange,
    /// # ) {
    /// let _ = root.deliver_performed_relational_change(phase, lowering, 0, change);
    /// # }
    /// ```
    #[doc(hidden)]
    pub fn deliver_performed_relational_change(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        lowering: &Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
        dependency_ordinal: usize,
        change: super::WorthQueryPerformedRelationalProductChange,
    ) -> Result<
        super::WorthQueryPerformedRelationalProductChangeDeliveryOutcome,
        super::WorthQueryPerformedRelationalProductChangeDeliveryDenial,
    > {
        use super::{
            WorthQueryPerformedRelationalProductChangeDeliveryDenial as Denial,
            WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as Kind,
        };
        let execution = match phase.request_for_owner(self.owner_identity) {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(Denial::new(
                    Kind::ExecutionRequest(cause.into()),
                    "delivery phase belongs to another installed runtime",
                    change,
                ))
            }
        };
        if !self.accepts_performed_change(&change) {
            return Err(Denial::new(
                Kind::ForeignProductRoot,
                "performed product change belongs to another application root",
                change,
            ));
        }
        let bridge = match self.bridge.upgrade() {
            Some(bridge) => bridge,
            None => {
                return Err(Denial::new(
                    Kind::BridgeRuntimeClosed,
                    "the sealed Bridge runtime is no longer live",
                    change,
                ));
            }
        };
        let bridge = bridge
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let signal_basis =
            match bridge.admit_conditional_signal_basis(lowering, change.signal_basis()) {
                Ok(signal_basis) => signal_basis,
                Err(denial) => {
                    return Err(Denial::new(
                        Kind::Bridge(denial.kind()),
                        denial.detail(),
                        change,
                    ))
                }
            };
        let patch = change.patch();
        let outcome = match bridge.deliver_authoritative_change(
            execution,
            &signal_basis,
            dependency_ordinal,
            patch,
        ) {
            Ok(outcome) => outcome,
            Err(denial) => {
                return Err(Denial::new(
                    Kind::Bridge(denial.kind()),
                    denial.detail(),
                    change,
                ));
            }
        };
        Ok(preserve_delivery_authority(outcome, change))
    }
}

pub(in crate::domain_computation) fn preserve_delivery_authority(
    outcome: worth_runtime_bridge::facade::CorrespondenceDeliveryOutcome,
    change: super::WorthQueryPerformedRelationalProductChange,
) -> super::WorthQueryPerformedRelationalProductChangeDeliveryOutcome {
    use super::WorthQueryPerformedRelationalProductChangeDeliveryOutcome as Outcome;
    use worth_proof::TransitionOutcome;
    match outcome {
        TransitionOutcome::Success(receipt) => Outcome::Success(receipt),
        TransitionOutcome::Denied(posture) => Outcome::Denied { posture, change },
        TransitionOutcome::Deferred(posture) => Outcome::Deferred { posture, change },
        TransitionOutcome::Stale(posture) => Outcome::Stale { posture, change },
        TransitionOutcome::RebindRequired(posture) => Outcome::RebindRequired { posture, change },
        TransitionOutcome::Failed(posture) => Outcome::Failed { posture, change },
    }
}
