#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiIntentOperabilityAppearanceClass {
    Ready,
    Pending,
    Occupied,
    Denied,
    Unsupported,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiIntentOperabilityStandingFact {
    graph_node: crate::graph::UiGraphNodeIdentity,
    /// The target the decision was observed for; condition re-observation
    /// admits it again before it refreshes the decision.
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    route: Box<str>,
    decision: super::UiIntentOperabilityDecision,
    class: UiIntentOperabilityAppearanceClass,
    owner_revision: u64,
}

impl UiIntentOperabilityStandingFact {
    pub(crate) fn seal(
        candidate: &super::super::payload::UiPreparedIntentPayload,
        decision: super::UiIntentOperabilityDecision,
        owner_revision: u64,
    ) -> Self {
        let target = candidate.input_basis().target();
        let class = appearance_class(decision.primary_cause().as_ref());
        Self {
            graph_node: candidate.graph_node(),
            target,
            route: candidate.declaration_identity().into(),
            decision,
            class,
            owner_revision,
        }
    }

    /// This fact with a re-observed decision, sealed at `owner_revision`.
    pub(in crate::runtime::intent) fn with_decision(
        &self,
        decision: super::UiIntentOperabilityDecision,
        owner_revision: u64,
    ) -> Self {
        Self {
            graph_node: self.graph_node,
            target: self.target,
            route: self.route.clone(),
            class: appearance_class(decision.primary_cause().as_ref()),
            decision,
            owner_revision,
        }
    }

    pub(crate) const fn class(&self) -> UiIntentOperabilityAppearanceClass {
        self.class
    }
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) const fn decision(&self) -> &super::UiIntentOperabilityDecision {
        &self.decision
    }
    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }
    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.target
    }
    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.target.mounted_instance()
    }
    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.target.node_receipt()
    }
    pub(in crate::runtime::intent) fn rebind_node_receipt(
        &mut self,
        receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) {
        self.target = self.target.with_node_receipt(receipt);
    }
    pub(in crate::runtime::intent) fn rebind_surface(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) {
        self.target = self.target.with_binding(binding);
    }
    pub(crate) fn route(&self) -> &str {
        &self.route
    }

    /// This fact as if it had been observed with `affinity`.
    #[cfg(test)]
    pub(crate) fn with_affinity_for_test(&self, affinity: super::UiIntentAffinityPosture) -> Self {
        self.with_decision(
            self.decision.with_affinity_for_test(affinity),
            self.owner_revision,
        )
    }

    /// This fact as if it had been recorded for `route`.
    #[cfg(test)]
    pub(crate) fn with_route_for_test(&self, route: &str) -> Self {
        Self {
            route: route.into(),
            ..self.clone()
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        route: &str,
        owner_revision: u64,
    ) -> Self {
        let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
            worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
            binding,
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
        );
        let target = crate::runtime::interaction::targeting::interaction_target_view_for_test(
            presentation,
            crate::mounting::UiMountedInteractionAffinityInput {
                surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
                binding,
                mounted_instance,
                node_receipt,
            },
        );
        Self {
            graph_node,
            target,
            route: route.into(),
            decision: super::UiIntentOperabilityDecision::ready_for_test(),
            class: UiIntentOperabilityAppearanceClass::Ready,
            owner_revision,
        }
    }
}

fn appearance_class(
    cause: Option<&super::UiIntentInoperableCause>,
) -> UiIntentOperabilityAppearanceClass {
    match cause {
        None => UiIntentOperabilityAppearanceClass::Ready,
        Some(super::UiIntentInoperableCause::Pending) => {
            UiIntentOperabilityAppearanceClass::Pending
        }
        Some(super::UiIntentInoperableCause::Occupied) => {
            UiIntentOperabilityAppearanceClass::Occupied
        }
        Some(super::UiIntentInoperableCause::Unsupported) => {
            UiIntentOperabilityAppearanceClass::Unsupported
        }
        Some(
            super::UiIntentInoperableCause::StaleTarget
            | super::UiIntentInoperableCause::WrongWorld
            | super::UiIntentInoperableCause::RebindRequired,
        ) => UiIntentOperabilityAppearanceClass::Stale,
        Some(super::UiIntentInoperableCause::ConditionWithheld { condition, .. }) => {
            withheld_class(condition.withholding())
        }
        Some(
            super::UiIntentInoperableCause::PolicyDenied
            | super::UiIntentInoperableCause::Readonly
            | super::UiIntentInoperableCause::ConfirmationRequired { .. },
        ) => UiIntentOperabilityAppearanceClass::Denied,
    }
}

/// An unavailable operand is usually transient, so it reads as pending; the
/// other withholdings keep their own posture.
const fn withheld_class(
    withholding: crate::runtime::expression::UiExpressionWithholding,
) -> UiIntentOperabilityAppearanceClass {
    match withholding {
        crate::runtime::expression::UiExpressionWithholding::Unavailable => {
            UiIntentOperabilityAppearanceClass::Pending
        }
        crate::runtime::expression::UiExpressionWithholding::Stale => {
            UiIntentOperabilityAppearanceClass::Stale
        }
        crate::runtime::expression::UiExpressionWithholding::Denied => {
            UiIntentOperabilityAppearanceClass::Denied
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{appearance_class, UiIntentOperabilityAppearanceClass as Class};
    use crate::declaration::UiIntentOperabilityDependencyAxis as Axis;
    use crate::runtime::expression::UiExpressionWithholding as Withholding;
    use crate::runtime::intent::UiIntentInoperableCause as Cause;

    #[test]
    fn closed_primary_cause_table_maps_every_owner_cause() {
        assert_eq!(appearance_class(None), Class::Ready);
        for (cause, expected) in [
            (Cause::Pending, Class::Pending),
            (Cause::Occupied, Class::Occupied),
            (Cause::Unsupported, Class::Unsupported),
            (Cause::StaleTarget, Class::Stale),
            (Cause::WrongWorld, Class::Stale),
            (Cause::RebindRequired, Class::Stale),
            (Cause::PolicyDenied, Class::Denied),
            (Cause::Readonly, Class::Denied),
            (
                Cause::ConfirmationRequired {
                    policy_identity: "confirm".into(),
                },
                Class::Denied,
            ),
            (withheld(Withholding::Unavailable), Class::Pending),
            (withheld(Withholding::Stale), Class::Stale),
            (withheld(Withholding::Denied), Class::Denied),
        ] {
            assert_eq!(appearance_class(Some(&cause)), expected);
        }
    }

    fn withheld(withholding: Withholding) -> Cause {
        let slot = crate::runtime::expression::UiExpressionSlot::for_test(0);
        Cause::ConditionWithheld {
            axis: Axis::Policy,
            condition: crate::runtime::intent::UiIntentWithheldCondition::new(slot, withholding),
        }
    }
}
