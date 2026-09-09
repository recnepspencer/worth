use super::*;
use crate::capability::*;
use crate::mounting::projection::semantic_text::{
    lower_semantic_text_formatting, lower_semantic_text_seed,
};
use std::collections::BTreeMap;

#[path = "text_geometry.rs"]
mod text_geometry;

#[test]
fn adopted_foreground_membership_follows_completed_candidate_visibility() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    adopt_value(&mut world, child);
    let visible = world.frame(&[0, 1], None);
    let original = context(&visible, child).text_foreground_spans;
    assert_eq!(original.len(), 1);
    assert_eq!(visible.appearance_text_candidates(child).unwrap().len(), 2);
    let (session, binding, _, vector, theme) = foreground_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let initial = sidecar
        .mount(lower(&context(&visible, child), &projection))
        .unwrap();
    let [UiMountedAppearanceMechanic::TextForeground(initial_foreground)] =
        initial.successor().mechanics()
    else {
        panic!("the adopted range must reach retained foreground lowering");
    };
    let identity =
        UiMountedAppearanceMechanic::TextForeground(initial_foreground.clone()).identity();

    // This ancestor intersects the Portal but lies beyond the text allocation.
    // Portal coverage alone must not retain the adopted foreground.
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(240_000, 40_000, 20_000, 20_000).unwrap()),
    );
    let hidden = world.frame(&[0, 1], None);
    let hidden_context = context(&hidden, child);
    assert!(matches!(
        hidden_context.appearance_clip(),
        Clip::Ancestor(_)
    ));
    assert!(hidden.appearance_text_candidates(child).unwrap().is_empty());
    assert!(hidden_context.text_foreground_spans().is_empty());
    assert_eq!(context(&visible, child).text_foreground_spans, original);
    let removal = sidecar.mount(lower(&hidden_context, &projection)).unwrap();
    assert!(removal.successor().mechanics().is_empty());
    assert_eq!(
        removal.changes(),
        &[worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(identity.clone())]
    );
    let rebuilt = sidecar
        .reconstruct(lower(&hidden_context, &projection))
        .unwrap();
    assert!(rebuilt.successor().mechanics().is_empty());

    // Partial coverage preserves the original range; it does not invent a new
    // span or infer glyph visibility from character offsets.
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(-10_000, -10_000, 110_000, 70_000).unwrap()),
    );
    let restored = world.frame(&[0, 1], None);
    assert_eq!(
        context(&restored, child).text_foreground_spans[0].identity(),
        original[0].identity(),
    );
    let candidates = restored.appearance_text_candidates(child).unwrap();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].clip_bounds().width(), 80.0);
    assert_eq!(candidates[0].clip_bounds().height(), 40.0);
    let restored_work = sidecar
        .mount(lower(&context(&restored, child), &projection))
        .unwrap();
    let [UiMountedAppearanceMechanic::TextForeground(restored_foreground)] =
        restored_work.successor().mechanics()
    else {
        panic!("visible candidate must restore its foreground");
    };
    assert_eq!(
        UiMountedAppearanceMechanic::TextForeground(restored_foreground.clone()).identity(),
        identity
    );
    assert_eq!(
        restored_work.changes(),
        &[
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Insert(
                UiMountedAppearanceMechanic::TextForeground(restored_foreground.clone())
            )
        ]
    );
}

fn foreground_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let aspect = UiAppearanceAspect::Foreground;
    let axis = UiAppearanceStateAxis::Validation;
    let contract = UiAppearanceAspectContract::component([aspect], []).unwrap();
    let partition = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(axis)])
        .with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::any(axis)]).uses_slot(
                UiThemeSlotIdentity::new("theme.appearance_consumer").unwrap(),
                UiThemeValueKind::Color,
            ),
        )
        .compile(aspect)
        .unwrap();
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.validation-foreground").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(aspect, partition)],
    )
    .unwrap()
}

fn foreground_inputs() -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::runtime::appearance::UiAppearanceNodeRoleBinding,
    crate::runtime::appearance::UiAppearanceTarget,
    crate::runtime::appearance::UiAppearanceStateVector,
    crate::runtime::appearance::UiThemeResolutionView,
) {
    use crate::runtime::tests::{
        appearance_component_session_test_support as support,
        appearance_theme_test_support as theme,
    };
    let role = foreground_role();
    let session = support::single_aspect_appearance_component_builder(
        &role,
        worth_ui_dsl::UiAppearanceAspect::Foreground,
    )
    .register_appearance_theme_bundle(theme::bundle())
    .unwrap()
    .with_rust_authored_declaration_fixture(support::appearance_fixture(&role))
    .freeze()
    .map(theme::activate)
    .unwrap()
    .launch()
    .unwrap();
    crate::runtime::appearance::projection_test_inputs_from_session(session, role)
}

#[test]
fn unavailable_text_clip_denies_context_instead_of_acknowledging_empty_paint() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    adopt_value(&mut world, child);
    let visible = world.frame(&[0, 1], None);
    let spans = context(&visible, child).text_foreground_spans;
    let denial = Denial::ScrollBindingUnavailable(
        world.semantic.node(child).unwrap().receipt().graph_node(),
    );
    world.set_clip(child, Clip::Unresolved(denial));
    let unavailable = world.frame(&[0, 1], None);
    assert_eq!(
        unavailable
            .appearance_visible_foreground_spans(child)
            .unwrap_err(),
        crate::mounting::projection::UiMountedAppearanceOutputDenial::AncestorClip(denial),
    );
    assert_eq!(
        unavailable
            .appearance_node_inputs_for_reconstruction()
            .err()
            .unwrap(),
        crate::mounting::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
    );
    assert_eq!(context(&visible, child).text_foreground_spans, spans);
}

#[test]
fn missing_retained_text_is_not_successful_clipping() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    adopt_value(&mut world, child);
    let visible = world.frame(&[0, 1], None);
    assert_eq!(context(&visible, child).text_foreground_spans().len(), 1);
    let mut missing = visible.clone();
    // Deliberately remove derived text while keeping admitted adoption. This is
    // corruption/recovery evidence, not a lawful authored suppression transition.
    missing.mechanics = Default::default();
    assert_eq!(
        missing
            .appearance_visible_foreground_spans(child)
            .unwrap_err(),
        crate::mounting::projection::UiMountedAppearanceOutputDenial::TextCandidate(
            crate::mounting::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
        ),
    );
    assert_eq!(context(&visible, child).text_foreground_spans().len(), 1);
}

fn adopt_value(world: &mut GeometryWorld, child: UiMountedInstanceIdentity) {
    let mut node = world.semantic.node(child).unwrap().clone();
    let graph_node = node.receipt().graph_node();
    let token = ThemeTokenId::new("theme.test.foreground").unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 220_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let span = ComponentSemanticTextSpanContract::new(
        worth_ui_host_contract::UiTextOriginalRange::new(0, 5).unwrap(),
        token.clone(),
        worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
    )
    .unwrap()
    .with_appearance_foreground();
    let contract = ComponentSemanticTextContract::spanned(token.clone(), 1, [span]).unwrap();
    let formatting = crate::mounting::UiMountedSemanticTextFormattingDirective::new(
        contract,
        BTreeMap::from([(
            token,
            crate::runtime::tests::appearance_component_session_test_support::appearance_theme_value(
                "#112233",
            ),
        )]),
    );
    let mut content = crate::mounting::UiMountedSemanticContentInput::empty();
    content
        .insert_scalar_with_formatting(
            graph_node,
            crate::mounting::UiMountedSemanticTextValueDirective::Replace(Arc::from("value")),
            Arc::from("CURRENT"),
            Some(formatting),
        )
        .unwrap();
    let input = content.get(graph_node);
    let formatting = lower_semantic_text_formatting(
        crate::mounting::UiMountedPlanProjectionSource::PreviewOnly,
        &crate::mounting::UiMountedThemeValueSource::ReplacementCandidateFrozenPlan,
        graph_node,
        None,
        input,
        None,
        false,
    )
    .unwrap();
    node.semantic_text = lower_semantic_text_seed(input, None, formatting).unwrap();
    world.semantic.insert_node(node);
}
