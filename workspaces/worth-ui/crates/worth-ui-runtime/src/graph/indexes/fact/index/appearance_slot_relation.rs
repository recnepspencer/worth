use std::collections::BTreeMap;

use crate::capability::{CapabilitySnapshot, ThemeTokenId};
use crate::fact_contract::UiConsumedFactContract;
use crate::graph::UiGraphSnapshot;

use super::super::{
    UiAuthoredDeclarationLookup, UiGraphFactConsumptionRelation, UiGraphFactIndexEntry,
};

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
