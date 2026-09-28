use std::collections::{BTreeMap, BTreeSet};

use crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority;
use crate::graph::{
    UiGraphFactConsumerIdentity, UiGraphFactConsumerKey, UiGraphFactConsumerKind, UiGraphSnapshot,
};
use crate::runtime::{WorthUiNodeLifecycleTransition, WorthUiNodeReplacementPlan};

use super::{UiIdentityLifecycleDecision, UiIdentityLifecycleDenial};

pub(crate) struct UiSourceIdentityLifecycleIndex {
    entries: BTreeMap<UiGraphFactConsumerKey, UiSourceIdentityLifecycleEvidence>,
}

struct UiSourceIdentityLifecycleEvidence {
    predecessor: Option<UiGraphFactConsumerIdentity>,
    candidate: Option<UiGraphFactConsumerIdentity>,
    transition: WorthUiNodeLifecycleTransition,
}

impl UiSourceIdentityLifecycleIndex {
    pub(crate) fn build(
        predecessor: &WorthUiPreparedApplicationAuthority,
        candidate: &WorthUiPreparedApplicationAuthority,
        plan: &WorthUiNodeReplacementPlan,
    ) -> Result<Self, UiIdentityLifecycleDenial> {
        let transitions = declaration_transitions(predecessor, candidate, plan)?;
        let mut entries = BTreeMap::new();
        record_snapshot(
            predecessor.graph_snapshot(),
            &transitions,
            true,
            &mut entries,
        )?;
        record_snapshot(
            candidate.graph_snapshot(),
            &transitions,
            false,
            &mut entries,
        )?;
        Ok(Self { entries })
    }

    pub(crate) fn selected_decision(
        &self,
        key: &UiGraphFactConsumerKey,
        predecessor: Option<UiGraphFactConsumerIdentity>,
        candidate: Option<UiGraphFactConsumerIdentity>,
    ) -> Result<UiIdentityLifecycleDecision, UiIdentityLifecycleDenial> {
        let evidence = self.entries.get(key).ok_or_else(|| {
            UiIdentityLifecycleDenial::MissingSelectedConsumer { key: key.clone() }
        })?;
        if evidence.predecessor != predecessor || evidence.candidate != candidate {
            return Err(
                UiIdentityLifecycleDenial::SelectedConsumerIdentityMismatch { key: key.clone() },
            );
        }
        selected_decision(key, evidence)
    }

    pub(crate) fn knows(&self, key: &UiGraphFactConsumerKey) -> bool {
        self.entries.contains_key(key)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn keys(&self) -> impl Iterator<Item = &UiGraphFactConsumerKey> {
        self.entries.keys()
    }
}

fn declaration_transitions(
    predecessor: &WorthUiPreparedApplicationAuthority,
    candidate: &WorthUiPreparedApplicationAuthority,
    plan: &WorthUiNodeReplacementPlan,
) -> Result<BTreeMap<Box<str>, WorthUiNodeLifecycleTransition>, UiIdentityLifecycleDenial> {
    let mut transitions = BTreeMap::new();
    for classification in plan.classifications() {
        let Some(provenance) = classification.authored_provenance_digest() else {
            continue;
        };
        let identities = declaration_identities_for_classification(
            predecessor,
            candidate,
            provenance,
            classification.transition(),
        );
        if identities.len() > 1 {
            return Err(UiIdentityLifecycleDenial::AmbiguousDeclarationProvenance {
                provenance_digest: provenance,
                declaration_count: identities.len(),
            });
        }
        let Some(identity) = identities.into_iter().next() else {
            continue;
        };
        insert_transition(&mut transitions, identity, classification.transition())?;
    }
    Ok(transitions)
}

fn declaration_identities_for_classification(
    predecessor: &WorthUiPreparedApplicationAuthority,
    candidate: &WorthUiPreparedApplicationAuthority,
    provenance: u64,
    transition: WorthUiNodeLifecycleTransition,
) -> BTreeSet<Box<str>> {
    match transition {
        WorthUiNodeLifecycleTransition::Drop => {
            declaration_identities_for_provenance(predecessor.graph_snapshot(), provenance)
        }
        WorthUiNodeLifecycleTransition::Create => {
            declaration_identities_for_provenance(candidate.graph_snapshot(), provenance)
        }
        WorthUiNodeLifecycleTransition::Preserve
        | WorthUiNodeLifecycleTransition::Replace
        | WorthUiNodeLifecycleTransition::Move
        | WorthUiNodeLifecycleTransition::Rebind
        | WorthUiNodeLifecycleTransition::LaneChange => [
            declaration_identities_for_provenance(predecessor.graph_snapshot(), provenance),
            declaration_identities_for_provenance(candidate.graph_snapshot(), provenance),
        ]
        .into_iter()
        .flatten()
        .collect(),
    }
}

fn declaration_identities_for_provenance(
    snapshot: &UiGraphSnapshot,
    provenance: u64,
) -> BTreeSet<Box<str>> {
    snapshot
        .graph_node_ids_for_authored_provenance(provenance)
        .iter()
        .filter_map(|node_identity| {
            snapshot
                .core_indexes()
                .declaration_correspondence()
                .declaration_identity_for(*node_identity)
                .map(|declaration| declaration.authored_semantic_name().into())
        })
        .collect()
}

fn insert_transition(
    transitions: &mut BTreeMap<Box<str>, WorthUiNodeLifecycleTransition>,
    identity: Box<str>,
    transition: WorthUiNodeLifecycleTransition,
) -> Result<(), UiIdentityLifecycleDenial> {
    if let Some(first) = transitions.insert(identity.clone(), transition) {
        if first != transition {
            return Err(
                UiIdentityLifecycleDenial::ConflictingDeclarationTransition {
                    authored_identity: identity,
                    first,
                    second: transition,
                },
            );
        }
    }
    Ok(())
}

fn record_snapshot(
    snapshot: &UiGraphSnapshot,
    transitions: &BTreeMap<Box<str>, WorthUiNodeLifecycleTransition>,
    predecessor: bool,
    entries: &mut BTreeMap<UiGraphFactConsumerKey, UiSourceIdentityLifecycleEvidence>,
) -> Result<(), UiIdentityLifecycleDenial> {
    for node in snapshot.nodes() {
        let identity: Box<str> = node.declaration_identity().authored_semantic_name().into();
        let transition = transitions
            .get(&identity)
            .copied()
            .unwrap_or(WorthUiNodeLifecycleTransition::Preserve);
        let repeated = node.repeated_instance_basis().identity_digest();
        record_consumer(
            UiGraphFactConsumerKey::new(
                UiGraphFactConsumerKind::GraphNode,
                identity.clone(),
                repeated,
            ),
            UiGraphFactConsumerIdentity::GraphNode(node.graph_node_identity()),
            transition,
            predecessor,
            entries,
        )?;
        if let Some(slot) = snapshot.mount_eligibility_slot_for_node(node.graph_node_identity()) {
            record_consumer(
                UiGraphFactConsumerKey::new(
                    UiGraphFactConsumerKind::MountEligibilitySlot,
                    identity,
                    repeated,
                ),
                UiGraphFactConsumerIdentity::MountEligibilitySlot(
                    slot.mount_eligibility_identity(),
                ),
                transition,
                predecessor,
                entries,
            )?;
        }
    }
    Ok(())
}

fn record_consumer(
    key: UiGraphFactConsumerKey,
    identity: UiGraphFactConsumerIdentity,
    transition: WorthUiNodeLifecycleTransition,
    predecessor: bool,
    entries: &mut BTreeMap<UiGraphFactConsumerKey, UiSourceIdentityLifecycleEvidence>,
) -> Result<(), UiIdentityLifecycleDenial> {
    let evidence = entries
        .entry(key.clone())
        .or_insert(UiSourceIdentityLifecycleEvidence {
            predecessor: None,
            candidate: None,
            transition,
        });
    if evidence.transition != transition {
        return Err(UiIdentityLifecycleDenial::ConflictingConsumerTransition { key });
    }
    if predecessor {
        evidence.predecessor = Some(identity);
    } else {
        evidence.candidate = Some(identity);
    }
    Ok(())
}

fn selected_decision(
    key: &UiGraphFactConsumerKey,
    evidence: &UiSourceIdentityLifecycleEvidence,
) -> Result<UiIdentityLifecycleDecision, UiIdentityLifecycleDenial> {
    let (predecessor, candidate) = (evidence.predecessor, evidence.candidate);
    decision_from_transition(
        key.kind(),
        evidence.transition,
        predecessor.is_some(),
        candidate.is_some(),
    )
    .ok_or_else(|| UiIdentityLifecycleDenial::ImpossibleSelectedTransition {
        key: key.clone(),
        transition: evidence.transition,
    })
}

pub(crate) const fn decision_from_transition(
    kind: UiGraphFactConsumerKind,
    transition: WorthUiNodeLifecycleTransition,
    has_predecessor: bool,
    has_candidate: bool,
) -> Option<UiIdentityLifecycleDecision> {
    match (has_predecessor, has_candidate) {
        (false, true) => Some(UiIdentityLifecycleDecision::Create),
        (true, false) => Some(UiIdentityLifecycleDecision::Retire),
        (true, true) => match transition {
            WorthUiNodeLifecycleTransition::Preserve => Some(UiIdentityLifecycleDecision::Preserve),
            WorthUiNodeLifecycleTransition::Move => Some(UiIdentityLifecycleDecision::Move),
            WorthUiNodeLifecycleTransition::Rebind => Some(UiIdentityLifecycleDecision::Rebind),
            WorthUiNodeLifecycleTransition::Replace
            | WorthUiNodeLifecycleTransition::LaneChange => match kind {
                UiGraphFactConsumerKind::GraphNode => Some(UiIdentityLifecycleDecision::Rebind),
                UiGraphFactConsumerKind::MountEligibilitySlot => {
                    Some(UiIdentityLifecycleDecision::Remount)
                }
            },
            WorthUiNodeLifecycleTransition::Drop | WorthUiNodeLifecycleTransition::Create => None,
        },
        (false, false) => None,
    }
}
