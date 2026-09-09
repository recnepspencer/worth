#[path = "validation/snapshot.rs"]
mod snapshot;
pub(crate) use snapshot::UiValidationAppearanceFactSnapshot;

#[cfg(test)]
#[path = "validation/owner_tests.rs"]
mod owner_tests;

type ValidationFacts = crate::runtime::persistent_index::UiPersistentOrdMap<
    worth_ui_host_contract::UiMountedInstanceIdentity,
    (
        crate::graph::UiGraphNodeIdentity,
        UiValidationAppearanceFact,
    ),
>;

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
    facts: ValidationFacts,
    #[allow(
        dead_code,
        reason = "Gate 0 assigns validation fact identities only in certification"
    )]
    next_identity: u64,
    revision: u64,
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
        let prior = owner
            .facts
            .get(&target.mounted_instance)
            .map(|(_, fact)| *fact);
        if prior.map(|fact| fact.revision) != expected_revision {
            return Err(UiValidationAppearanceFactDenial::StalePredecessor);
        }
        if prior.is_some_and(|fact| fact.class == class && fact.node_receipt == node_receipt) {
            return Ok(());
        }
        let identity = prior.map_or(owner.next_identity, |fact| fact.identity);
        let next_identity = if prior.is_none() {
            owner
                .next_identity
                .checked_add(1)
                .ok_or(UiValidationAppearanceFactDenial::IdentityExhausted)?
        } else {
            owner.next_identity
        };
        let revision = match prior {
            None => 1,
            Some(fact) => fact
                .revision
                .checked_add(1)
                .ok_or(UiValidationAppearanceFactDenial::RevisionExhausted)?,
        };
        let owner_revision = owner
            .revision
            .checked_add(1)
            .ok_or(UiValidationAppearanceFactDenial::RevisionExhausted)?;
        owner.facts.insert(
            target.mounted_instance,
            (
                target.graph_node,
                UiValidationAppearanceFact {
                    identity,
                    revision,
                    node_receipt,
                    class,
                },
            ),
        );
        owner.next_identity = next_identity;
        owner.revision = owner_revision;
        Ok(())
    }

    pub(crate) fn retire_validation_appearance_instance(
        &mut self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) {
        let Some(owner) = self.validation_owner.as_mut() else {
            return;
        };
        if owner.facts.get(&mounted_instance).is_some() {
            let revision = owner
                .revision
                .checked_add(1)
                .expect("bounded validation revision exhausted");
            owner.facts.remove(&mounted_instance);
            owner.revision = revision;
        }
    }

    pub(crate) fn validation_appearance_snapshot(
        &self,
    ) -> Option<UiValidationAppearanceFactSnapshot> {
        self.validation_owner
            .as_ref()
            .map(|owner| UiValidationAppearanceFactSnapshot {
                owner_revision: owner.revision,
                facts: owner.facts.clone(),
            })
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
        session
            .mounted_graph_node(graph_node)
            .map_err(|_| UiValidationAppearanceTargetAdmissionDenial::UnknownGraphNode)?;
        let basis = session
            .current_mounted_identity_basis(mounted_instance)
            .ok_or(UiValidationAppearanceTargetAdmissionDenial::Mounted(
                crate::mounting::UiMountedIdentityDenial::UnknownMountedInstance,
            ))?;
        if basis.graph_node_identity() != graph_node {
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
