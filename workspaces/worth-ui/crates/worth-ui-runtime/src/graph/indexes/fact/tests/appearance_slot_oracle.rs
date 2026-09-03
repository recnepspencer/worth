use std::collections::BTreeSet;

const ALIASED_SLOT: &str = "theme.pulse.alias";
const TERMINAL_SLOT: &str = super::STATIC_PAINT_TOKEN;
const ALIAS_AUTHORED_IDENTITY: &str = "token:theme.pulse.alias";
const TERMINAL_AUTHORED_IDENTITY: &str = "token:theme.pulse.static";
const ATTACHED_COMPONENT: &str = "pulse.component.alias";
const PEER_COMPONENT: &str = "pulse.component.alias.peer";

#[test]
fn canonical_slot_selection_rebuild_matches_authoritative_graph_fact_oracle_for_aliases() {
    let mut app = aliased_role_only_app();
    let expected = {
        let authority = app.prepared_authority();
        let declarations = authority.authored_declaration_lookup();
        assert_eq!(
            declarations.theme_token_declaration_identity(ALIASED_SLOT),
            Some(ALIAS_AUTHORED_IDENTITY)
        );
        assert_eq!(
            declarations.theme_token_declaration_identity(TERMINAL_SLOT),
            Some(TERMINAL_AUTHORED_IDENTITY)
        );
        (
            authoritative_role_slot_consumers(
                authority.graph_snapshot(),
                authority.capabilities(),
                &declarations,
                ALIASED_SLOT,
            ),
            authoritative_role_slot_consumers(
                authority.graph_snapshot(),
                authority.capabilities(),
                &declarations,
                TERMINAL_SLOT,
            ),
        )
    };
    assert_eq!(expected.0, expected.1);
    assert_eq!(expected.0.len(), 1);

    let before_rebuild = {
        let authority = app.prepared_authority();
        let declarations = authority.authored_declaration_lookup();
        (
            observed_slot_consumers(authority.consumed_fact_index(), &declarations, ALIASED_SLOT),
            observed_slot_consumers(
                authority.consumed_fact_index(),
                &declarations,
                TERMINAL_SLOT,
            ),
        )
    };
    assert_eq!(before_rebuild, expected);

    app.rebuild_prepared_derived_indexes();

    let after_rebuild = {
        let authority = app.prepared_authority();
        let declarations = authority.authored_declaration_lookup();
        (
            observed_slot_consumers(authority.consumed_fact_index(), &declarations, ALIASED_SLOT),
            observed_slot_consumers(
                authority.consumed_fact_index(),
                &declarations,
                TERMINAL_SLOT,
            ),
        )
    };
    assert_eq!(after_rebuild, expected);
}

fn observed_slot_consumers(
    index: &crate::graph::UiGraphConsumedFactIndex,
    declarations: &crate::graph::UiAuthoredDeclarationLookup,
    capability_identity: &str,
) -> BTreeSet<crate::graph::UiGraphNodeIdentity> {
    let authored_identity = declarations
        .theme_token_declaration_identity(capability_identity)
        .unwrap_or(capability_identity);
    let selection = crate::runtime::appearance::UiAppearanceConsumerSelection::try_for_slot(
        index,
        capability_identity,
        authored_identity,
    )
    .expect("declared theme slot should resolve");
    assert!(selection.is_reconstructible());
    selection.consumers().iter().copied().collect()
}

fn authoritative_role_slot_consumers(
    snapshot: &crate::graph::UiGraphSnapshot,
    capabilities: &crate::capability::CapabilitySnapshot,
    declarations: &crate::graph::UiAuthoredDeclarationLookup,
    capability_identity: &str,
) -> BTreeSet<crate::graph::UiGraphNodeIdentity> {
    let authored_identity = declarations
        .theme_token_declaration_identity(capability_identity)
        .unwrap_or(capability_identity);
    snapshot
        .nodes()
        .iter()
        .filter_map(|node| {
            let attachment = node.appearance_role_attachment()?;
            if node.component_reference() != Some(attachment.target()) {
                return None;
            }
            let role = capabilities.appearance_roles().get(attachment.role())?;
            if role.aspect_contract() != attachment.aspect_contract()
                || role.revision() != attachment.revision()
            {
                return None;
            }
            role.slot_uses()
                .iter()
                .any(|slot_use| {
                    let requested_slot = slot_use.slot().as_str();
                    let matches = |candidate: &str| {
                        candidate == capability_identity || candidate == authored_identity
                    };
                    matches(requested_slot)
                        || authoritative_terminal_slot(capabilities, requested_slot)
                            .as_deref()
                            .is_some_and(|terminal| matches(terminal))
                })
                .then_some(node.graph_node_identity())
        })
        .collect()
}

fn authoritative_terminal_slot(
    capabilities: &crate::capability::CapabilitySnapshot,
    requested_slot: &str,
) -> Option<Box<str>> {
    let requested_id = crate::capability::ThemeTokenId::new(requested_slot).ok()?;
    let terminal = capabilities
        .theme_tokens()
        .get_entry(&requested_id)?
        .resolved_target_id()
        .as_str();
    (terminal != requested_slot).then(|| terminal.to_owned().into_boxed_str())
}

fn aliased_role_only_app() -> crate::facade::WorthUiApp {
    crate::declaration::appearance_fact_index_test_support::aliased_role_only_fact_index_app(
        "aliased-role-only-fact-index",
        ALIASED_SLOT,
        TERMINAL_SLOT,
        ALIAS_AUTHORED_IDENTITY,
        TERMINAL_AUTHORED_IDENTITY,
        ATTACHED_COMPONENT,
        PEER_COMPONENT,
    )
}
