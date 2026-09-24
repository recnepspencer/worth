pub(crate) struct UiIntentPostureObservation {
    graph_node: crate::graph::UiGraphNodeIdentity,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    reference: crate::fact_contract::UiIntentPostureReference,
    posture: crate::fact_contract::UiIntentPostureKind,
    owner_order: u64,
}

pub(crate) struct UiIntentPostureCommit {
    owner_order: u64,
}

impl UiIntentPostureObservation {
    pub(super) const fn new(
        graph_node: crate::graph::UiGraphNodeIdentity,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
        reference: crate::fact_contract::UiIntentPostureReference,
        posture: crate::fact_contract::UiIntentPostureKind,
        owner_order: u64,
    ) -> (Self, UiIntentPostureCommit) {
        (
            Self {
                graph_node,
                target,
                reference,
                posture,
                owner_order,
            },
            UiIntentPostureCommit { owner_order },
        )
    }

    pub(crate) const fn owner_order(&self) -> u64 {
        self.owner_order
    }

    pub(super) const fn with_owner_order(self, owner_order: u64) -> (Self, UiIntentPostureCommit) {
        Self::new(
            self.graph_node,
            self.target,
            self.reference,
            self.posture,
            owner_order,
        )
    }

    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.target.mounted_instance()
    }

    pub(crate) const fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    pub(crate) const fn into_parts(
        self,
    ) -> (
        crate::graph::UiGraphNodeIdentity,
        crate::runtime::interaction::UiPresentedInteractionTargetView,
        crate::fact_contract::UiIntentPostureReference,
        crate::fact_contract::UiIntentPostureKind,
    ) {
        (self.graph_node, self.target, self.reference, self.posture)
    }
}

impl UiIntentPostureCommit {
    pub(super) const fn owner_order(&self) -> u64 {
        self.owner_order
    }
}
