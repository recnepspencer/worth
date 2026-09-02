#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Gate 0 freezes validation classes before product publication"
)]
pub(crate) enum UiValidationAppearanceClass {
    Valid,
    Advisory,
    Invalid,
    Pending,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiValidationAppearanceFact {
    identity: u64,
    revision: u64,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    class: UiValidationAppearanceClass,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct UiValidationAppearanceTarget {
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
}

#[allow(
    dead_code,
    reason = "Gate 0 retains owner-admitted validation targets for certification"
)]
pub(crate) struct UiAdmittedValidationAppearanceTarget {
    target: UiValidationAppearanceTarget,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Gate 0 freezes validation target admission denials"
)]
pub(crate) enum UiValidationAppearanceTargetAdmissionDenial {
    UnknownGraphNode,
    Mounted(crate::mounting::UiMountedIdentityDenial),
    InstanceDoesNotBelongToGraphNode,
}

pub(super) struct UiValidationAppearanceOwner {
    facts: std::collections::BTreeMap<UiValidationAppearanceTarget, UiValidationAppearanceFact>,
    #[allow(
        dead_code,
        reason = "Gate 0 assigns validation fact identities only in certification"
    )]
    next_identity: u64,
    revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiValidationAppearanceFactSnapshot {
    owner_revision: u64,
    facts: Box<[(UiValidationAppearanceTarget, UiValidationAppearanceFact)]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Gate 0 freezes validation publication denials without emitting"
)]
pub(crate) enum UiValidationAppearanceFactDenial {
    OwnerUnavailable,
    StalePredecessor,
    IdentityExhausted,
    RevisionExhausted,
}

impl super::UiIntentApplicationFactState {
    #[allow(
        dead_code,
        reason = "Gate 0 exercises validation publication only in certification"
    )]
    pub(crate) fn publish_validation_appearance_fact(
        &mut self,
        target: UiAdmittedValidationAppearanceTarget,
        expected_revision: Option<u64>,
        class: UiValidationAppearanceClass,
    ) -> Result<(), UiValidationAppearanceFactDenial> {
        let owner = self
            .validation_owner
            .as_mut()
            .ok_or(UiValidationAppearanceFactDenial::OwnerUnavailable)?;
        let UiAdmittedValidationAppearanceTarget {
            target,
            node_receipt,
        } = target;
        let prior = owner.facts.get(&target).copied();
        if prior.map(|fact| fact.revision) != expected_revision {
            return Err(UiValidationAppearanceFactDenial::StalePredecessor);
        }
        let identity = prior.map_or_else(
            || {
                let identity = owner.next_identity;
                owner.next_identity = identity
                    .checked_add(1)
                    .ok_or(UiValidationAppearanceFactDenial::IdentityExhausted)?;
                Ok(identity)
            },
            |fact| Ok(fact.identity),
        )?;
        let revision = match prior {
            None => 1,
            Some(fact) => fact
                .revision
                .checked_add(1)
                .ok_or(UiValidationAppearanceFactDenial::RevisionExhausted)?,
        };
        owner.revision = owner
            .revision
            .checked_add(1)
            .ok_or(UiValidationAppearanceFactDenial::RevisionExhausted)?;
        owner.facts.insert(
            target,
            UiValidationAppearanceFact {
                identity,
                revision,
                node_receipt,
                class,
            },
        );
        Ok(())
    }

    pub(crate) fn retire_validation_appearance_instance(
        &mut self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) {
        let Some(owner) = self.validation_owner.as_mut() else {
            return;
        };
        let before = owner.facts.len();
        owner
            .facts
            .retain(|target, _| target.mounted_instance != mounted_instance);
        if owner.facts.len() != before {
            owner.revision = owner
                .revision
                .checked_add(1)
                .expect("bounded validation revision exhausted");
        }
    }

    pub(crate) fn validation_appearance_snapshot(
        &self,
    ) -> Option<UiValidationAppearanceFactSnapshot> {
        self.validation_owner
            .as_ref()
            .map(|owner| UiValidationAppearanceFactSnapshot {
                owner_revision: owner.revision,
                facts: self
                    .validation_owner
                    .as_ref()
                    .expect("validation owner is present")
                    .facts
                    .iter()
                    .map(|(target, fact)| (*target, *fact))
                    .collect(),
            })
    }
}

impl UiValidationAppearanceFactSnapshot {
    #[cfg(test)]
    pub(crate) fn for_test(
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        class: UiValidationAppearanceClass,
    ) -> Self {
        Self {
            owner_revision: 1,
            facts: Box::new([(
                UiValidationAppearanceTarget {
                    graph_node,
                    mounted_instance,
                },
                UiValidationAppearanceFact {
                    identity: 1,
                    revision: 1,
                    node_receipt,
                    class,
                },
            )]),
        }
    }
}

impl UiAdmittedValidationAppearanceTarget {
    #[allow(
        dead_code,
        reason = "Gate 0 exercises target admission only in certification"
    )]
    pub(crate) fn admit(
        session: &crate::facade::WorthUiActiveApplicationSession,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Result<Self, UiValidationAppearanceTargetAdmissionDenial> {
        let node = session
            .mounted_graph_node(graph_node)
            .map_err(|_| UiValidationAppearanceTargetAdmissionDenial::UnknownGraphNode)?;
        let instances = session
            .mounted_instances_for(node)
            .map_err(UiValidationAppearanceTargetAdmissionDenial::Mounted)?;
        if !instances.contains(&mounted_instance) {
            return Err(
                UiValidationAppearanceTargetAdmissionDenial::InstanceDoesNotBelongToGraphNode,
            );
        }
        session
            .validate_current_mounted_node_receipt(mounted_instance, node_receipt)
            .map_err(UiValidationAppearanceTargetAdmissionDenial::Mounted)?;
        Ok(Self {
            target: UiValidationAppearanceTarget {
                graph_node,
                mounted_instance,
            },
            node_receipt,
        })
    }
}

impl UiValidationAppearanceOwner {
    pub(super) fn new() -> Self {
        Self {
            facts: Default::default(),
            next_identity: 1,
            revision: 0,
        }
    }
}

impl UiValidationAppearanceFactSnapshot {
    #[allow(
        dead_code,
        reason = "Gate 0 exposes validation owner revision for future consumers"
    )]
    pub(crate) const fn owner_revision(&self) -> u64 {
        self.owner_revision
    }

    #[allow(
        dead_code,
        reason = "Gate 0 exposes validation classification without live appearance"
    )]
    pub(crate) fn class_for(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Option<UiValidationAppearanceClass> {
        let (_, fact) = self.facts.iter().find(|(target, _)| {
            target.graph_node == graph_node && target.mounted_instance == mounted_instance
        })?;
        Some(if fact.node_receipt == node_receipt {
            fact.class
        } else {
            UiValidationAppearanceClass::Stale
        })
    }

    pub(crate) fn fact_basis_for(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<(
        u64,
        u64,
        worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    )> {
        self.facts.iter().find_map(|(target, fact)| {
            (target.graph_node == graph_node && target.mounted_instance == mounted_instance)
                .then_some((fact.identity, fact.revision, fact.node_receipt))
        })
    }

    #[cfg(test)]
    pub(crate) fn fact_count(&self) -> usize {
        self.facts.len()
    }
}
