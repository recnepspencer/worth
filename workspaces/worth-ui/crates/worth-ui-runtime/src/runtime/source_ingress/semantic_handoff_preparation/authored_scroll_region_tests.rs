use std::path::PathBuf;

use worth_ui_dsl::{WorthUiAuthoredSourceInput, WorthUiDslCompiler, WorthUiSealedSemanticPackage};

use crate::capability::{
    MosaicChildRule, MosaicClippingPosture, MosaicFocusScopeKind, MosaicHitTestPosture,
    MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicRegionPersistence, MosaicRegionRole,
    MosaicScrollOwnership, MosaicSizingBehavior, ThemeTokenId, UiAuthoredScrollRegionCause,
    UiScrollAxisSupport, UiScrollChromeContract, UiScrollChromeContractDenial, UiScrollLineExtent,
};
use crate::facade::entry::WorthUiHostNeutralApp;
use crate::facade::WorthUi;

use super::{prepare_semantic_handoff, WorthUiSemanticHandoffPreparationStop};

const REGION: &str = "test.region.scrolled_list";
const TRACK_ROLE: &str = "test.appearance.scroll_track";
const THUMB_ROLE: &str = "test.appearance.scroll_thumb";

/// The authored `line_extent` reaches the descriptor of the region the block
/// names, and the snapshot it was lowered from still states its own truth.
#[test]
fn an_authored_line_extent_reaches_the_region_descriptor_it_names() {
    let app = scroll_region_app(RustScrollClauses::none());
    let material = prepare_semantic_handoff(scroll_package("line_extent 20"), app.capabilities())
        .expect("an authored line extent for a registered region admits");
    let (_, _, evidence) = material.into_parts();
    let successor = evidence
        .successor_snapshot()
        .expect("an authored clause the registry does not yet state owes a successor");

    assert_eq!(
        successor
            .mosaic_regions()
            .get(&region_id())
            .expect("the named region stays registered")
            .scroll_line_extent(),
        Some(line_extent(20))
    );
    assert_eq!(
        app.capabilities()
            .mosaic_regions()
            .get(&region_id())
            .expect("the predecessor still states its own truth")
            .scroll_line_extent(),
        None
    );
    assert_ne!(successor.digest(), app.capabilities().digest());
}

/// The authored `chrome`, `track` and `thumb` reach the same descriptor as one
/// contract, carrying the axes and both painted roles the source named.
#[test]
fn authored_chrome_reaches_the_region_descriptor_as_one_contract() {
    let app = scroll_region_app(RustScrollClauses::none());
    let material = prepare_semantic_handoff(
        scroll_package(&format!(
            "chrome both track {TRACK_ROLE} thumb {THUMB_ROLE}"
        )),
        app.capabilities(),
    )
    .expect("an authored chrome contract for a registered region admits");
    let (_, _, evidence) = material.into_parts();
    let successor = evidence
        .successor_snapshot()
        .expect("an authored clause the registry does not yet state owes a successor");

    assert_eq!(
        successor
            .mosaic_regions()
            .get(&region_id())
            .expect("the named region stays registered")
            .scroll_chrome(),
        Some(&chrome_contract(UiScrollAxisSupport::Both))
    );
}

/// Two sources stating the same answer is agreement, not succession: the frozen
/// snapshot already states the whole truth and nothing is rebuilt.
#[test]
fn a_source_that_agrees_with_the_registered_descriptor_owes_no_successor() {
    let app = scroll_region_app(RustScrollClauses {
        line_extent: Some(20),
        chrome: Some(UiScrollAxisSupport::Block),
    });
    let material = prepare_semantic_handoff(
        scroll_package(&format!(
            "line_extent 20 chrome block track {TRACK_ROLE} thumb {THUMB_ROLE}"
        )),
        app.capabilities(),
    )
    .expect("a source that repeats the registered answer admits");
    let (_, _, evidence) = material.into_parts();

    assert_eq!(evidence.successor_snapshot(), None);
}

/// Two sources naming one region with two different line extents is a denial.
/// Neither the authored number nor the registered one silently wins.
#[test]
fn a_line_extent_disagreement_on_one_region_is_denied() {
    let app = scroll_region_app(RustScrollClauses {
        line_extent: Some(24),
        chrome: None,
    });

    assert_eq!(
        refused(scroll_package("line_extent 20"), &app),
        UiAuthoredScrollRegionCause::LineExtentDisagreement {
            authored: 20,
            registered: 24,
        }
    );
}

/// The same rule holds for chrome: one region, two contracts, no precedence.
#[test]
fn a_chrome_disagreement_on_one_region_is_denied() {
    let app = scroll_region_app(RustScrollClauses {
        line_extent: None,
        chrome: Some(UiScrollAxisSupport::Block),
    });

    assert_eq!(
        refused(
            scroll_package(&format!(
                "chrome both track {TRACK_ROLE} thumb {THUMB_ROLE}"
            )),
            &app,
        ),
        UiAuthoredScrollRegionCause::ChromeDisagreement
    );
}

/// A clause that names a region kind nobody registered describes a region that
/// does not exist, and is refused before any successor is built.
#[test]
fn an_authored_clause_cannot_name_an_unregistered_region() {
    let app = scroll_region_app(RustScrollClauses::none());
    let package = WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            "scroll test.region.absent { anchor clamp line_extent 20 }",
        ),
    )
    .expect("scroll source seals");

    assert_eq!(
        refused(package, &app),
        UiAuthoredScrollRegionCause::UnregisteredRegion
    );
}

/// A `scroll` block that states no region clause declares an application-wide
/// policy only, so its identity is never looked up as a region kind.
#[test]
fn a_policy_only_scroll_block_names_no_region_descriptor() {
    let app = scroll_region_app(RustScrollClauses::none());
    let package = WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            "scroll results_list { nested anchor clamp }",
        ),
    )
    .expect("scroll source seals");

    let material = prepare_semantic_handoff(package, app.capabilities())
        .expect("a policy-only scroll declaration names no region kind");
    let (_, _, evidence) = material.into_parts();

    assert_eq!(evidence.successor_snapshot(), None);
}

/// Chrome names two painted roles. This is where those names meet the
/// appearance role registry, because the contract itself holds no registry.
#[test]
fn authored_chrome_cannot_name_an_unregistered_appearance_role() {
    let app = scroll_region_app(RustScrollClauses::none());

    assert_eq!(
        refused(
            scroll_package(&format!(
                "chrome both track test.appearance.absent thumb {THUMB_ROLE}"
            )),
            &app,
        ),
        UiAuthoredScrollRegionCause::Chrome(UiScrollChromeContractDenial::UnregisteredRole)
    );
}

/// Chrome on a region that owns no scrollable content would reserve a gutter
/// for a bar that can never travel.
#[test]
fn authored_chrome_cannot_reach_a_region_that_owns_no_scrolling() {
    let app = unscrolled_region_app();

    assert_eq!(
        refused(
            scroll_package(&format!(
                "chrome both track {TRACK_ROLE} thumb {THUMB_ROLE}"
            )),
            &app,
        ),
        UiAuthoredScrollRegionCause::Chrome(UiScrollChromeContractDenial::AxisNotOwned)
    );
}

fn refused(
    package: WorthUiSealedSemanticPackage,
    app: &WorthUiHostNeutralApp,
) -> UiAuthoredScrollRegionCause {
    let denial = match prepare_semantic_handoff(package, app.capabilities()) {
        Ok(_) => panic!("the authored scroll clauses must not reach a descriptor"),
        Err(denial) => denial,
    };
    let WorthUiSemanticHandoffPreparationStop::AuthoredScrollRegion(refusal) = denial.stop() else {
        panic!("the stop must name the authored scroll clause that was refused")
    };
    assert_eq!(refusal.declaration_index(), 0);
    refusal.cause()
}

/// What the Rust registration states about the fixture region's scroll clauses.
struct RustScrollClauses {
    line_extent: Option<u16>,
    chrome: Option<UiScrollAxisSupport>,
}

impl RustScrollClauses {
    const fn none() -> Self {
        Self {
            line_extent: None,
            chrome: None,
        }
    }
}

fn region_id() -> MosaicRegionKindId {
    MosaicRegionKindId::new(REGION).expect("the fixture region id is well formed")
}

fn line_extent(points: u16) -> UiScrollLineExtent {
    UiScrollLineExtent::logical_points(points).expect("the fixture line extent is admissible")
}

fn chrome_contract(axes: UiScrollAxisSupport) -> UiScrollChromeContract {
    UiScrollChromeContract::new(axes, role_identity(TRACK_ROLE), role_identity(THUMB_ROLE))
        .expect("the fixture track and thumb are two distinct roles")
}

fn role_identity(identity: &str) -> worth_ui_dsl::UiAppearanceRoleIdentity {
    worth_ui_dsl::UiAppearanceRoleIdentity::new(identity)
        .expect("the fixture role identity is well formed")
}

fn scroll_package(clauses: &str) -> WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace")).with_module(
            "app/main.wui",
            format!("scroll {REGION} {{ nested anchor stable_key {clauses} }}"),
        ),
    )
    .expect("scroll source seals")
}

fn scroll_region_app(rust: RustScrollClauses) -> WorthUiHostNeutralApp {
    let descriptor = region_descriptor(MosaicScrollOwnership::region_owned());
    let descriptor = match rust.line_extent {
        Some(points) => descriptor.with_scroll_line_extent(line_extent(points)),
        None => descriptor,
    };
    let descriptor = match rust.chrome {
        Some(axes) => descriptor.with_scroll_chrome(chrome_contract(axes)),
        None => descriptor,
    };
    paintable_app(descriptor)
}

fn unscrolled_region_app() -> WorthUiHostNeutralApp {
    paintable_app(region_descriptor(MosaicScrollOwnership::no_scrolling()))
}

fn region_descriptor(ownership: MosaicScrollOwnership) -> MosaicRegionKindDescriptor {
    MosaicRegionKindDescriptor::new(region_id(), MosaicRegionRole::auxiliary())
        .with_sizing_behavior(MosaicSizingBehavior::viewport_bounded())
        .with_scroll_ownership(ownership)
        .with_focus_scope(MosaicFocusScopeKind::region_scope())
        .with_child_rule(MosaicChildRule::leaf_only())
        .with_persistence(MosaicRegionPersistence::restorable())
        .with_clipping(MosaicClippingPosture::clip_to_region())
        .with_hit_test(MosaicHitTestPosture::pass_through())
}

fn paintable_app(descriptor: MosaicRegionKindDescriptor) -> WorthUiHostNeutralApp {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_mosaic_region_kind(descriptor)
        .register_appearance_role(chrome_role(TRACK_ROLE, "test.scroll.track_slot"))
        .expect("the fixture track role registers")
        .register_appearance_role(chrome_role(THUMB_ROLE, "test.scroll.thumb_slot"))
        .expect("the fixture thumb role registers")
        .register_theme_token(theme_token("test.scroll.track_slot"))
        .register_theme_token(theme_token("test.scroll.thumb_slot"))
        .freeze()
        .expect("the scroll region fixture app freezes")
}

fn theme_token(slot: &str) -> crate::capability::ThemeTokenDescriptor {
    crate::runtime::tests::appearance_component_session_test_support::appearance_theme_token(
        ThemeTokenId::new(slot).expect("the fixture theme token id is well formed"),
    )
}

fn chrome_role(identity: &str, slot: &str) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .expect("a background-only aspect contract is admissible");
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
        .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(slot).expect("the fixture slot is well formed"),
            worth_ui_dsl::UiThemeValueKind::Color,
        ))
        .compile(worth_ui_dsl::UiAppearanceAspect::Background)
        .expect("a single-cell background partition compiles");
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        role_identity(identity),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).expect("revision one is admissible"),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
    )
    .expect("the fixture chrome role is admissible")
}
