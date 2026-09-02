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
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    route: Box<str>,
    decision: super::UiIntentOperabilityDecision,
    class: UiIntentOperabilityAppearanceClass,
    owner_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiIntentOperabilityStandingFactSnapshot {
    owner_revision: u64,
    facts: Box<[UiIntentOperabilityStandingFact]>,
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
            mounted_instance: target.mounted_instance(),
            node_receipt: target.node_receipt(),
            route: candidate.declaration_identity().into(),
            decision,
            class,
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
    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub(crate) fn route(&self) -> &str {
        &self.route
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        route: &str,
        owner_revision: u64,
    ) -> Self {
        Self {
            graph_node,
            mounted_instance,
            node_receipt,
            route: route.into(),
            decision: super::UiIntentOperabilityDecision::ready_for_test(),
            class: UiIntentOperabilityAppearanceClass::Ready,
            owner_revision,
        }
    }
}

impl UiIntentOperabilityStandingFactSnapshot {
    pub(crate) fn seal(owner_revision: u64, facts: Vec<UiIntentOperabilityStandingFact>) -> Self {
        Self {
            owner_revision,
            facts: facts.into_boxed_slice(),
        }
    }

    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }
    pub(crate) fn facts(&self) -> &[UiIntentOperabilityStandingFact] {
        &self.facts
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
        Some(
            super::UiIntentInoperableCause::PolicyDenied
            | super::UiIntentInoperableCause::Readonly
            | super::UiIntentInoperableCause::ConfirmationRequired { .. },
        ) => UiIntentOperabilityAppearanceClass::Denied,
    }
}

#[cfg(test)]
mod tests {
    use super::{appearance_class, UiIntentOperabilityAppearanceClass as Class};
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
        ] {
            assert_eq!(appearance_class(Some(&cause)), expected);
        }
    }
}
