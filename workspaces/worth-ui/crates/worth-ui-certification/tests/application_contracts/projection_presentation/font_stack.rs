use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::{
    app::{
        UiApplicationFontFaceDefinition, UiApplicationFontLicenseRecord,
        UiApplicationFontPackDefinition, UiFontCollectionGeneration, UiFontFamilyStack,
        UiFontSlant, UiGlobalFontCollection, UiTextFaceRequest, UiTextOriginalRange, UiTextStyle,
        UiTextStyleInput,
    },
    declaration::{
        ComponentSemanticTextContract, ComponentSemanticTextSpanContract, ThemeColorValue,
        ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
    },
    observation::UiChangeClassificationOutcome,
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome},
};
use worth_ui_dsl::WorthUiRustAuthoredArtifactInput;
use worth_ui_host_contract::{UiSemanticTextSlot, UiTextCoverageDisposition};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::UiProjectionObservation;

use super::scalar_query_only::{
    component_descriptor, mount_and_allocate, projection_module_with_additional_token,
    scalar_registration, status_region_descriptor, text_token_descriptor, ACTIVE_COMPONENT,
    CANDIDATE_COMPONENT,
};
use crate::projection_lifecycle::support::ScalarLifecycleWorld;

#[path = "font_stack/accessibility_geometry.rs"]
mod accessibility_geometry;

const VALUE: &str = "Ready \u{21AF}\n\u{5E9}\u{5DC}\u{5D5}\u{5DD} \u{1F469}\u{200D}\u{1F4BB}";
const ACCENT_COLOR: &str = "theme.platform_pulse.projected_status.accent_text";

#[test]
fn authored_application_stack_and_emoji_fallback_cross_mounted_headless_consumers() {
    let (
        fonts,
        _pack,
        primary_family,
        secondary_family,
        primary_face,
        _primary_bytes,
        secondary_face,
        _secondary_bytes,
    ) = application_fonts();
    let primary_style = style([primary_family, secondary_family]);
    let secondary_style = style([secondary_family, primary_family]);
    let emoji_style = primary_style.clone();
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 320.0,
            height: 96.0,
        },
    );
    let (mut query, completion) = ScalarLifecycleWorld::standard(NodeId::new(41_410, 0), VALUE);
    let registration = scalar_registration(&query);
    let component = component_descriptor(ACTIVE_COMPONENT).with_semantic_text(
        ComponentSemanticTextContract::spanned(
            ThemeTokenId::new(super::scalar_query_only::TEXT_COLOR).unwrap(),
            1,
            [
                span(0, 6, super::scalar_query_only::TEXT_COLOR, primary_style),
                span(6, 10, ACCENT_COLOR, secondary_style),
                span(10, VALUE.len() as u32, ACCENT_COLOR, emoji_style),
            ],
        )
        .unwrap(),
    );
    let app = worth_ui::facade::app::WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .with_font_collection(Arc::new(fonts))
        .register_component(component)
        .register_component(component_descriptor(CANDIDATE_COMPONENT))
        .register_mosaic_region_kind(status_region_descriptor())
        .register_theme_token(text_token_descriptor())
        .register_theme_token(accent_token_descriptor())
        .register_scalar_projection(registration)
        .unwrap()
        .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([
            projection_module_with_additional_token(ACTIVE_COMPONENT, ACCENT_COLOR, "#f7812f"),
        ]))
        .freeze()
        .map(|application| {
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_recorder(
                application,
                recorder.clone(),
            )
        })
        .unwrap();
    let mut session = app.launch().unwrap();
    mount_and_allocate(&mut session);

    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(UiProjectionObservation::Scalar(
        current.into_fact_and_predecessor().0.into_observation(),
    ))
    .unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("font-stack text must publish"),
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(414))
        .unwrap();
    assert!(matches!(prepared.execute(1), UiRebindOutcome::Published(_)));

    let transcripts = recorder.observed_transcripts();
    let value = transcripts[0]
        .semantic_text()
        .iter()
        .find(|row| row.slot() == UiSemanticTextSlot::Value)
        .unwrap();
    assert_eq!(value.text(), VALUE);
    assert_eq!(
        value
            .styles()
            .iter()
            .map(|style| style.original_range())
            .collect::<Vec<_>>(),
        [
            UiTextOriginalRange::new(0, 6).unwrap(),
            UiTextOriginalRange::new(6, 10).unwrap(),
            UiTextOriginalRange::new(10, VALUE.len() as u32).unwrap(),
        ]
    );
    assert_eq!(value.foregrounds().len(), 3);
    assert_eq!(
        value.foregrounds()[0].color().channels(),
        [255, 255, 255, 255]
    );
    assert_eq!(
        value.foregrounds()[1].color().channels(),
        [247, 129, 47, 255]
    );
    assert_eq!(
        value.foregrounds()[2].color().channels(),
        [247, 129, 47, 255]
    );
    assert_eq!(value.font_collection_generation().get(), 2);
    assert_eq!(
        value.qualified_measurement().layout_identity(),
        value.layout_identity()
    );
    assert_eq!(
        value.qualified_measurement().logical_bounds(),
        value.logical_bounds()
    );
    assert_eq!(
        value.qualified_measurement().ink_bounds(),
        value.ink_bounds()
    );
    assert_eq!(
        value.accessibility_geometry().layout_identity(),
        value.layout_identity()
    );
    accessibility_geometry::assert_exact_multiline_bidi_records(&value.accessibility_geometry());
    assert!(
        value.qualified_layout_cost().coverage_index_queries()
            >= value.qualified_layout_cost().face_shape_attempts(),
        "the mounted artifact must carry the exact coverage-index work that preceded shaping"
    );

    let latin = value
        .coverage()
        .iter()
        .find(|coverage| coverage.original_range().start() == 0)
        .unwrap();
    assert_eq!(latin.face(), Some(primary_face));
    let symbol_start = VALUE.find('\u{21AF}').unwrap() as u32;
    let symbol_coverage = value
        .coverage()
        .iter()
        .find(|coverage| coverage.original_range().start() == symbol_start)
        .unwrap();
    assert_eq!(symbol_coverage.face(), Some(secondary_face));
    let emoji_start = VALUE.find('\u{1F469}').unwrap() as u32;
    let emoji_coverage = value
        .coverage()
        .iter()
        .find(|coverage| coverage.original_range().start() == emoji_start)
        .unwrap();
    assert_eq!(
        emoji_coverage.disposition(),
        UiTextCoverageDisposition::QualifiedFace
    );
    assert!(![primary_face, secondary_face].contains(&emoji_coverage.face().unwrap()));
    assert_eq!(
        value
            .graphemes()
            .iter()
            .filter(|record| record.original_range().start() == emoji_start)
            .count(),
        1,
        "the complete ZWJ emoji must stay one grapheme"
    );
}

fn style(families: [worth_ui::facade::app::UiQualifiedFontFamilyIdentity; 2]) -> UiTextStyle {
    UiTextStyle::new(UiTextStyleInput {
        language: Arc::from("und"),
        font_size_millipoints: 14_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        family_stack: UiFontFamilyStack::new(Box::new(families)).unwrap(),
        face_request: UiTextFaceRequest::regular(),
        features: Box::new([]),
        variations: Box::new([]),
    })
    .unwrap()
}

fn span(
    start: u32,
    end: u32,
    token: &str,
    style: UiTextStyle,
) -> ComponentSemanticTextSpanContract {
    ComponentSemanticTextSpanContract::new(
        UiTextOriginalRange::new(start, end).unwrap(),
        ThemeTokenId::new(token).unwrap(),
        style,
    )
    .unwrap()
}

fn accent_token_descriptor() -> ThemeTokenDescriptor {
    ThemeTokenDescriptor::define(
        ThemeTokenId::new(ACCENT_COLOR).unwrap(),
        ThemeTokenFamily::surface(),
        ThemeTokenSource::application(),
        ThemeTokenValue::color(ThemeColorValue::hex("#f7812f").unwrap()),
    )
}

fn application_fonts() -> (
    UiGlobalFontCollection,
    worth_ui::facade::app::UiQualifiedFontPackIdentity,
    worth_ui::facade::app::UiQualifiedFontFamilyIdentity,
    worth_ui::facade::app::UiQualifiedFontFamilyIdentity,
    worth_ui::facade::app::UiQualifiedFontFaceIdentity,
    Arc<[u8]>,
    worth_ui::facade::app::UiQualifiedFontFaceIdentity,
    Arc<[u8]>,
) {
    let (profile, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let primary_bytes: Arc<[u8]> = Arc::from(
        include_bytes!("../../../../worth-ui-host-native/assets/fonts/NotoSans-Regular.ttf")
            .as_slice(),
    );
    let secondary_bytes: Arc<[u8]> = Arc::from(
        include_bytes!(
            "../../../../../profiles/worth-ui-global-text-v2/fonts/NotoSansSymbols2-Regular.ttf"
        )
        .as_slice(),
    );
    let definition = UiApplicationFontPackDefinition {
        name: Arc::from("mounted application typography"),
        faces: Box::new([
            face("Application Primary", Arc::clone(&primary_bytes)),
            face("Application Symbols", Arc::clone(&secondary_bytes)),
        ]),
    };
    let (collection, receipt, _) = profile
        .register_application_pack(UiFontCollectionGeneration::new(2).unwrap(), definition)
        .unwrap();
    let family = receipt.family("Application Primary").unwrap();
    let secondary = receipt.family("Application Symbols").unwrap();
    let selected = receipt
        .faces()
        .iter()
        .find(|face| face.family() == family)
        .unwrap()
        .identity();
    let secondary_selected = receipt
        .faces()
        .iter()
        .find(|face| face.family() == secondary)
        .unwrap()
        .identity();
    (
        collection,
        receipt.identity(),
        family,
        secondary,
        selected,
        primary_bytes,
        secondary_selected,
        secondary_bytes,
    )
}

fn face(family: &str, bytes: Arc<[u8]>) -> UiApplicationFontFaceDefinition {
    UiApplicationFontFaceDefinition {
        family: Arc::from(family),
        bytes,
        face_index: 0,
        weight: 400,
        width_milli_percent: 100_000,
        slant: UiFontSlant::Upright,
        license: UiApplicationFontLicenseRecord {
            identifier: Arc::from("OFL-1.1"),
            notice: Arc::from("Repository-pinned test font bytes."),
        },
    }
}
