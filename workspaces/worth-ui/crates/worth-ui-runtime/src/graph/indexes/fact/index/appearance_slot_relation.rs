use std::collections::BTreeMap;

use crate::capability::{CapabilitySnapshot, ThemeTokenId};
use crate::fact_contract::UiConsumedFactContract;
use crate::graph::UiGraphSnapshot;

use super::super::{UiAuthoredDeclarationLookup, UiGraphFactIndexEntry};

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
            add_slot_consumer(
                snapshot,
                node,
                slot_use.slot().as_str(),
                authored_declarations,
                by_declaration,
            );
            if let Some(terminal) = terminal_slot(capabilities, slot_use.slot().as_str()) {
                if &*terminal != slot_use.slot().as_str() {
                    add_slot_consumer(
                        snapshot,
                        node,
                        &terminal,
                        authored_declarations,
                        by_declaration,
                    );
                }
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
) {
    let authored_identity: Box<str> = authored_declarations
        .theme_token_declaration_identity(capability_identity)
        .unwrap_or(capability_identity)
        .into();
    let contract = UiConsumedFactContract::authored(authored_identity.clone());
    let entries = by_declaration.entry(authored_identity).or_default();
    if entries.iter().any(|entry| {
        matches!(
            entry.consumer(),
            crate::graph::UiGraphFactConsumerIdentity::GraphNode(identity)
                if identity == node.graph_node_identity()
        )
    }) {
        return;
    }
    super::push_component_consumer(entries, snapshot, node, contract, None);
}

fn terminal_slot(capabilities: &CapabilitySnapshot, requested: &str) -> Option<Box<str>> {
    let themes = capabilities.appearance_themes()?;
    let requested = ThemeTokenId::new(requested).ok()?;
    themes
        .catalog()
        .resolved_target(&requested)
        .map(|terminal| terminal.as_str().to_owned().into_boxed_str())
}
