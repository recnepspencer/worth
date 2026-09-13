use std::collections::BTreeMap;

use crate::capability::{CapabilitySnapshot, ThemeTokenId};
use crate::fact_contract::UiConsumedFactContract;
use crate::graph::UiGraphSnapshot;

use super::super::{
    UiAuthoredDeclarationLookup, UiGraphFactConsumptionRelation, UiGraphFactIndexBasis,
    UiGraphFactIndexEntry, UiGraphFactLookupDenial,
};

impl super::UiGraphConsumedFactIndex {
    pub(crate) fn select_appearance_slot_consumers(
        &self,
        requested_basis: UiGraphFactIndexBasis,
        capability_identity: &str,
        authored_identity: &str,
    ) -> Result<UiGraphAppearanceSlotSelection, UiGraphFactLookupDenial> {
        if requested_basis != self.basis() {
            return Err(UiGraphFactLookupDenial::BasisMismatch {
                index_basis: self.basis(),
                requested_basis,
            });
        }
        if !self
            .appearance_theme_slots
            .admits(capability_identity, authored_identity)
        {
            return Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration {
                authored_identity: authored_identity.into(),
            });
        }
        let entries = self
            .authored_by_declaration
            .get(authored_identity)
            .map_or(&[][..], |entries| entries.as_ref());
        let mut consumers = entries
            .iter()
            .filter(|entry| {
                entry
                    .consumption_relation()
                    .matches_theme_token(capability_identity, authored_identity)
            })
            .filter_map(|entry| match entry.consumer() {
                crate::graph::UiGraphFactConsumerIdentity::GraphNode(node) => Some(node),
                crate::graph::UiGraphFactConsumerIdentity::MountEligibilitySlot(_) => None,
            })
            .collect::<Vec<_>>();
        consumers.sort_unstable();
        consumers.dedup();
        Ok(UiGraphAppearanceSlotSelection {
            consumers: consumers.into_boxed_slice(),
            entries_examined: entries.len(),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UiGraphAppearanceSlotSelection {
    consumers: Box<[crate::graph::UiGraphNodeIdentity]>,
    entries_examined: usize,
}

impl UiGraphAppearanceSlotSelection {
    pub(crate) fn consumers(&self) -> &[crate::graph::UiGraphNodeIdentity] {
        &self.consumers
    }
    /// One capability-domain lookup and one authored consumer-index lookup.
    pub(crate) const fn index_probes(&self) -> usize {
        2
    }
    pub(crate) const fn entries_examined(&self) -> usize {
        self.entries_examined
    }
    pub(crate) fn into_consumers(self) -> Box<[crate::graph::UiGraphNodeIdentity]> {
        self.consumers
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiGraphAppearanceThemeSlotDomain {
    authored_by_capability: BTreeMap<Box<str>, Box<str>>,
}

impl UiGraphAppearanceThemeSlotDomain {
    pub(super) fn from_capabilities(
        capabilities: &CapabilitySnapshot,
        authored_declarations: &UiAuthoredDeclarationLookup,
    ) -> Self {
        let authored_by_capability = capabilities
            .theme_tokens()
            .entries()
            .iter()
            .map(|entry| entry.descriptor().id().as_str())
            .chain(
                capabilities
                    .appearance_themes()
                    .into_iter()
                    .flat_map(|themes| {
                        themes
                            .catalog()
                            .slots()
                            .map(|slot| slot.identity().as_str())
                    }),
            )
            .map(|capability_identity| {
                let authored_identity = authored_declarations
                    .theme_token_declaration_identity(capability_identity)
                    .unwrap_or(capability_identity);
                (
                    capability_identity.into(),
                    authored_identity.to_owned().into_boxed_str(),
                )
            })
            .collect();
        Self {
            authored_by_capability,
        }
    }

    fn admits(&self, capability_identity: &str, authored_identity: &str) -> bool {
        self.authored_by_capability
            .get(capability_identity)
            .is_some_and(|admitted| admitted.as_ref() == authored_identity)
    }
}

pub(super) fn add_role_slot_consumers(
    snapshot: &UiGraphSnapshot,
    capabilities: &CapabilitySnapshot,
    authored_declarations: &UiAuthoredDeclarationLookup,
    by_declaration: &mut BTreeMap<Box<str>, Vec<UiGraphFactIndexEntry>>,
) {
    for node in snapshot.nodes() {
        let Some(attachment) = node.appearance_role_attachment() else {
            continue;
        };
        if node.component_reference() != Some(attachment.target()) {
            continue;
        }
        let Some(role) = capabilities.appearance_roles().get(attachment.role()) else {
            continue;
        };
        if role.aspect_contract() != attachment.aspect_contract()
            || role.revision() != attachment.revision()
        {
            continue;
        }
        for slot_use in role.slot_uses() {
            let terminal = terminal_slot(capabilities, slot_use.slot().as_str());
            let relation = UiGraphFactConsumptionRelation::appearance_role_slot(
                role.role().clone(),
                role.revision(),
                slot_use.aspect(),
                slot_use.slot().as_str(),
                terminal.clone(),
            );
            add_slot_consumer(
                snapshot,
                node,
                slot_use.slot().as_str(),
                authored_declarations,
                by_declaration,
                relation.clone(),
            );
            if terminal
                .as_deref()
                .is_some_and(|terminal| terminal != slot_use.slot().as_str())
            {
                add_slot_consumer(
                    snapshot,
                    node,
                    terminal
                        .as_deref()
                        .expect("alias terminal was just checked"),
                    authored_declarations,
                    by_declaration,
                    relation,
                );
            }
        }
    }
}

fn add_slot_consumer(
    snapshot: &UiGraphSnapshot,
    node: &crate::graph::UiGraphNode,
    capability_identity: &str,
    authored_declarations: &UiAuthoredDeclarationLookup,
    by_declaration: &mut BTreeMap<Box<str>, Vec<UiGraphFactIndexEntry>>,
    relation: UiGraphFactConsumptionRelation,
) {
    let authored_identity: Box<str> = authored_declarations
        .theme_token_declaration_identity(capability_identity)
        .unwrap_or(capability_identity)
        .into();
    let contract = UiConsumedFactContract::authored(authored_identity.clone());
    let entries = by_declaration.entry(authored_identity).or_default();
    super::push_component_consumer(entries, snapshot, node, contract, relation);
}

fn terminal_slot(capabilities: &CapabilitySnapshot, requested: &str) -> Option<Box<str>> {
    let requested = ThemeTokenId::new(requested).ok()?;
    capabilities
        .theme_tokens()
        .get_entry(&requested)
        .map(|entry| entry.resolved_target_id())
        .filter(|terminal| *terminal != &requested)
        .map(|terminal| terminal.as_str().to_owned().into_boxed_str())
}
