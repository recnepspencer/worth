use std::sync::Arc;

use super::super::WorthQueryProductSharedRoot;

impl WorthQueryProductSharedRoot {
    #[doc(hidden)]
    pub fn deliver_performed_relational_change(
        &self,
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
                    Kind::Bridge,
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
                Err(denial) => return Err(Denial::new(Kind::Bridge, denial.detail(), change)),
            };
        let patch = change.patch();
        let outcome =
            match bridge.deliver_authoritative_change(&signal_basis, dependency_ordinal, patch) {
                Ok(outcome) => outcome,
                Err(denial) => {
                    return Err(Denial::new(Kind::Bridge, denial.detail(), change));
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
