use std::collections::BTreeSet;

use worth_ui_dsl::{
    UiDslAspectName, UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey,
    UiDslSourceProvenance, UiDslStructuralToken,
};

#[path = "tests/appearance_attachment.rs"]
mod appearance_attachment;
#[path = "tests/appearance_slot_oracle.rs"]
mod appearance_slot_oracle;
#[path = "tests/appearance_state.rs"]
mod appearance_state;

use crate::capability::{
    ThemeColorValue, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue,
};
use crate::facade::{WorthUi, WorthUiRustAuthoredDeclarationFixture};
use crate::fact_contract::{
    UiAuthoredChangedFact, UiAuthoredFactKind, UiAuthoredFactSelector,
    UiHostDeviceScaleChangedFact, UiProducedFact,
};
use crate::graph::{
    UiGraphConsumedFactIndex, UiGraphFactConsumerIdentity, UiGraphFactLookupDenial,
    UiGraphSessionLabel, UiGraphWorldProfile,
};

const PUBLISHER: &str = "pulse.identity.publisher";
const CONSUMER: &str = "pulse.identity.consumer";
const STATIC_PAINT_COMPONENT: &str = "pulse.component.static";
const STATIC_PAINT_PEER: &str = "pulse.component.static.peer";
const STATIC_PAINT_TOKEN: &str = "theme.pulse.static";

#[test]
fn authored_lookup_joins_direct_identity_and_declared_aspect_consumers() {
    let app = indexed_app("fact-index-join");
    let authority = app.prepared_authority();
    let index = authority.consumed_fact_index();
    let snapshot = authority.graph_snapshot();
    let publisher_node = graph_node_for(snapshot, PUBLISHER);
    let consumer_node = graph_node_for(snapshot, CONSUMER);
    let publisher_slot = snapshot
        .mount_eligibility_slot_for_node(publisher_node)
        .expect("publisher should have one mount-eligibility slot")
        .mount_eligibility_identity();
    let consumer_slot = snapshot
        .mount_eligibility_slot_for_node(consumer_node)
        .expect("consumer should have one mount-eligibility slot")
        .mount_eligibility_identity();
    let fact = UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
        UiAuthoredFactSelector::node(PUBLISHER),
        UiAuthoredFactKind::SemanticsChanged,
    ));

    let receipt = index
        .lookup(index.basis(), &fact)
        .expect("exact authored declaration should resolve");
    let observed = receipt
        .entries()
        .iter()
        .map(|entry| entry.consumer())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        observed,
        BTreeSet::from([
            UiGraphFactConsumerIdentity::GraphNode(publisher_node),
            UiGraphFactConsumerIdentity::MountEligibilitySlot(publisher_slot),
            UiGraphFactConsumerIdentity::GraphNode(consumer_node),
            UiGraphFactConsumerIdentity::MountEligibilitySlot(consumer_slot),
        ])
    );
    assert_eq!(receipt.cost().index_probes(), 1);
    assert_eq!(receipt.cost().contract_checks(), 4);
    assert_eq!(receipt.cost().selected_consumers(), 4);
    assert_eq!(
        receipt
            .entries()
            .iter()
            .filter(|entry| entry.affected_aspect().is_some())
            .count(),
        2
    );
}

#[test]
fn subsystem_lookup_selects_only_declared_matching_aspect_family() {
    let app = indexed_app("fact-index-subsystem");
    let index = app.prepared_authority().consumed_fact_index();
    let device_scale =
        UiProducedFact::HostDeviceScale(UiHostDeviceScaleChangedFact::new(2_000_000));

    let receipt = index
        .lookup(index.basis(), &device_scale)
        .expect("device-scale family should resolve through appearance consumption");

    assert_eq!(receipt.entries().len(), 2);
    assert!(receipt.entries().iter().all(|entry| {
        entry
            .affected_aspect()
            .is_some_and(|aspect| aspect.canonical_label() == "appearance.background")
    }));
}

#[test]
fn rebuild_is_deterministic_and_foreign_basis_is_rejected() {
    let left = indexed_app("fact-index-left");
    let right = foreign_indexed_app();
    let left_authority = left.prepared_authority();
    let original = left_authority.consumed_fact_index();
    let rebuilt = UiGraphConsumedFactIndex::rebuild(
        left_authority.graph_snapshot(),
        left_authority.capabilities(),
        &left_authority.authored_declaration_lookup(),
        left_authority.semantic_handoff().projection_contents(),
    );
    let fact = UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
        UiAuthoredFactSelector::node(PUBLISHER),
        UiAuthoredFactKind::SemanticsChanged,
    ));

    assert_eq!(*original, rebuilt);
    assert_ne!(
        original.basis(),
        right.prepared_authority().consumed_fact_index().basis(),
        "the stale-basis control must be issued by a genuinely different graph world"
    );
    assert_eq!(
        original.lookup(
            right.prepared_authority().consumed_fact_index().basis(),
            &fact
        ),
        Err(UiGraphFactLookupDenial::BasisMismatch {
            index_basis: original.basis(),
            requested_basis: right.prepared_authority().consumed_fact_index().basis(),
        })
    );
}

fn foreign_indexed_app() -> crate::facade::WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(UiGraphWorldProfile::preview_session_label(
            UiGraphSessionLabel::new("fact-index-foreign")
                .expect("foreign graph session label should admit"),
        ))
        .with_rust_authored_declaration_fixture(
            WorthUiRustAuthoredDeclarationFixture::named("fact-index-foreign")
                .with_semantic_artifact_spec(publisher_spec())
                .with_semantic_artifact_spec(content_consumer_spec())
                .with_semantic_artifact_spec(appearance_consumer_spec()),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("foreign fact-index fixture should prepare")
}

#[test]
fn unknown_authored_identity_denies_without_fallback_scan() {
    let app = indexed_app("fact-index-miss");
    let index = app.prepared_authority().consumed_fact_index();
    let fact = UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
        UiAuthoredFactSelector::node("pulse.identity.missing"),
        UiAuthoredFactKind::SemanticsChanged,
    ));

    assert_eq!(
        index.lookup(index.basis(), &fact),
        Err(UiGraphFactLookupDenial::UnknownAuthoredDeclaration {
            authored_identity: "pulse.identity.missing".into(),
        })
    );
}

#[test]
fn prepared_rebuild_reconstructs_nonempty_aspect_and_fact_projections() {
    let mut app = indexed_app("fact-index-prepared-rebuild");
    let before_published = app
        .prepared_authority()
        .graph_snapshot()
        .core_indexes()
        .published_aspects()
        .clone();
    let before_consumed = app
        .prepared_authority()
        .graph_snapshot()
        .core_indexes()
        .consumed_aspects()
        .clone();
    let before_facts = app.prepared_authority().consumed_fact_index().clone();
    assert_eq!(
        before_published
            .iter()
            .map(|(aspect, _)| aspect.canonical_label())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["content.text", "structure.product-root"])
    );
    assert_eq!(
        before_consumed
            .iter()
            .map(|(aspect, _)| aspect.canonical_label())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["appearance.background", "content.text"])
    );
    assert!(!before_facts
        .lookup(
            before_facts.basis(),
            &UiProducedFact::AuthoredSource(UiAuthoredChangedFact::new(
                UiAuthoredFactSelector::node(PUBLISHER),
                UiAuthoredFactKind::SemanticsChanged,
            )),
        )
        .expect("non-empty authored fact projection should resolve")
        .entries()
        .is_empty());

    app.rebuild_prepared_derived_indexes();

    assert_eq!(
        before_published,
        *app.prepared_authority()
            .graph_snapshot()
            .core_indexes()
            .published_aspects()
    );
    assert_eq!(
        before_consumed,
        *app.prepared_authority()
            .graph_snapshot()
            .core_indexes()
            .consumed_aspects()
    );
    assert_eq!(
        before_facts,
        *app.prepared_authority().consumed_fact_index()
    );
}

fn indexed_app(package: &str) -> crate::facade::WorthUiApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_rust_authored_declaration_fixture(
            WorthUiRustAuthoredDeclarationFixture::named(package)
                .with_semantic_artifact_spec(publisher_spec())
                .with_semantic_artifact_spec(content_consumer_spec())
                .with_semantic_artifact_spec(appearance_consumer_spec()),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("fact-index fixture should prepare")
}

fn static_paint_app() -> crate::facade::WorthUiApp {
    let token = ThemeTokenId::new(STATIC_PAINT_TOKEN).unwrap();
    let role = crate::runtime::tests::appearance_component_session_test_support::validation_background_role(
        STATIC_PAINT_TOKEN,
    );
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_component(
            crate::runtime::tests::appearance_component_session_test_support::static_paint_component(
                STATIC_PAINT_COMPONENT,
                token.clone(),
            ),
        )
        .register_appearance_role(role.clone())
        .unwrap()
        .register_theme_token(ThemeTokenDescriptor::define(
            token,
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(ThemeColorValue::hex("#112233").unwrap()),
        ))
        .with_rust_authored_declaration_fixture(
            WorthUiRustAuthoredDeclarationFixture::named("static-paint-fact-index")
                .with_semantic_artifact_spec(
                    UiDslSemanticArtifactSpec::new(
                        UiDslSemanticKey::new(STATIC_PAINT_COMPONENT),
                        UiDslSemanticFamily::Control,
                        UiDslSourceProvenance::file_authored("app/static.wui", 0),
                    )
                    .with_structural_token(UiDslStructuralToken::new("control:static-paint"))
                    .with_component_reference(
                        worth_ui_dsl::UiDslComponentReference::new(STATIC_PAINT_COMPONENT).unwrap(),
                    )
                    .unwrap()
                    .with_appearance_role_attachment(
                        worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                            role.role().clone(),
                            role.revision(),
                        ),
                    )
                    .unwrap(),
                )
                .with_semantic_artifact_spec(
                    UiDslSemanticArtifactSpec::new(
                        UiDslSemanticKey::new(STATIC_PAINT_PEER),
                        UiDslSemanticFamily::Control,
                        UiDslSourceProvenance::file_authored("app/static.wui", 1),
                    )
                    .with_structural_token(UiDslStructuralToken::new(
                        "control:static-paint-peer",
                    ))
                    .with_component_reference(
                        worth_ui_dsl::UiDslComponentReference::new(STATIC_PAINT_COMPONENT).unwrap(),
                    )
                    .unwrap(),
                )
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("static-paint fact-index fixture should prepare")
}

fn publisher_spec() -> UiDslSemanticArtifactSpec {
    UiDslSemanticArtifactSpec::new(
        UiDslSemanticKey::new(PUBLISHER),
        UiDslSemanticFamily::Control,
        UiDslSourceProvenance::file_authored("app/fact_index.wui", 0),
    )
    .with_structural_token(UiDslStructuralToken::new("control:publisher"))
    .with_published_aspect(UiDslAspectName::new("content.text"))
}

fn content_consumer_spec() -> UiDslSemanticArtifactSpec {
    UiDslSemanticArtifactSpec::new(
        UiDslSemanticKey::new(CONSUMER),
        UiDslSemanticFamily::Region,
        UiDslSourceProvenance::file_authored("app/fact_index.wui", 1),
    )
    .with_structural_token(UiDslStructuralToken::new("region:content-consumer"))
    .with_consumed_aspect(UiDslAspectName::new("content.text"))
}

fn appearance_consumer_spec() -> UiDslSemanticArtifactSpec {
    UiDslSemanticArtifactSpec::new(
        UiDslSemanticKey::new("pulse.appearance.consumer"),
        UiDslSemanticFamily::Region,
        UiDslSourceProvenance::file_authored("app/fact_index.wui", 2),
    )
    .with_structural_token(UiDslStructuralToken::new("region:appearance-consumer"))
    .with_consumed_aspect(UiDslAspectName::new("appearance.background"))
}

fn graph_node_for(
    snapshot: &crate::graph::UiGraphSnapshot,
    authored_identity: &str,
) -> crate::graph::UiGraphNodeIdentity {
    snapshot
        .nodes()
        .iter()
        .find(|node| node.declaration_identity().authored_semantic_name() == authored_identity)
        .map(crate::graph::UiGraphNode::graph_node_identity)
        .unwrap_or_else(|| panic!("expected graph node for {authored_identity}"))
}

fn graph_node_named<'snapshot>(
    snapshot: &'snapshot crate::graph::UiGraphSnapshot,
    authored_identity: &str,
) -> &'snapshot crate::graph::UiGraphNode {
    snapshot
        .nodes()
        .iter()
        .find(|node| node.declaration_identity().authored_semantic_name() == authored_identity)
        .unwrap_or_else(|| panic!("expected graph node for {authored_identity}"))
}
