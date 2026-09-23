use super::*;
use crate::capability::*;
#[path = "adopted_text.rs"]
mod adopted_text;
use adopted_text::adopt_value;

#[path = "text_geometry.rs"]
mod text_geometry;

#[test]
fn ordinary_ancestor_clipping_reaches_text_commands_and_visual_inspection() {
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    adopt_value(&mut world, child);
    let mut node = world.semantic.node(child).unwrap().clone();
    node.portal_child_owner = None;
    world.semantic.insert_node(node);
    // Ordinary text occupies [8,12,220,120]. This viewport lies fully inside
    // it; its visible extent must therefore be exactly 50x30, without Portal.
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(30_000, 20_000, 50_000, 30_000).unwrap()),
    );
    let clipped = world.frame(&[], None);
    assert_presented_text_clip(&clipped, child, Some([50.0, 30.0]));
    world.set_clip(child, Clip::Suppressed);
    let hidden = world.frame(&[], None);
    assert_presented_text_clip(&hidden, child, None);
}

#[test]
fn adopted_foreground_survives_clipping_but_retires_with_suppressed_content() {
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

    // Empty ancestor coverage retains images and their paint for Scroll reveal.
    world.set_clip(
        child,
        Clip::Ancestor(UiAppearanceClip::new(240_000, 40_000, 20_000, 20_000).unwrap()),
    );
    let clipped = world.frame(&[0, 1], None);
    let candidates = clipped.appearance_text_candidates(child).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates
        .iter()
        .all(|row| row.clip_bounds().width() == 0.0));
    let clipped_spans = context(&clipped, child).text_foreground_spans;
    assert_eq!(clipped_spans.len(), original.len());
    assert_eq!(clipped_spans[0].identity(), original[0].identity());
    assert_ne!(
        clipped_spans, original,
        "coverage changed, not span identity"
    );
    // Explicit suppression removes content, rather than merely clipping it.
    world.set_clip(child, Clip::Suppressed);
    let hidden = world.frame(&[0, 1], None);
    let hidden_context = context(&hidden, child);
    assert!(matches!(hidden_context.appearance_clip, Clip::Suppressed));
    assert!(hidden.appearance_text_candidates(child).unwrap().is_empty());
    assert_presented_text_clip(&hidden, child, None);
    assert!(hidden_context.text_foreground_spans.is_empty());
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
    assert_presented_text_clip(&restored, child, Some([80.0, 40.0]));
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

fn assert_presented_text_clip(
    frame: &UiMountedProjectionFrame,
    instance: UiMountedInstanceIdentity,
    expected: Option<[f32; 2]>,
) {
    let node = frame.semantic.node(instance).unwrap();
    let surface = frame
        .semantic
        .surface_for(node.receipt.semantic_surface())
        .unwrap();
    let rows = frame.semantic_text_view_rows(surface).unwrap().rows;
    let ordinary = rows
        .iter()
        .filter(|row| row.mounted_instance() == instance)
        .collect::<Vec<_>>();
    let commands =
        frame.presentation_commands_for_instance(instance, surface.surface, surface.binding);
    let paint = commands
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic),
            _ => None,
        })
        .collect::<Vec<_>>();
    // Source commands retain their original layout-work accounting; reused
    // projection rows clear it. Compare the actual presented attribution.
    let attribution = |row: &&worth_ui_host_contract::UiMountedSemanticTextMechanic| {
        (
            row.node_receipt(),
            row.bounds(),
            row.clip_bounds(),
            row.semantic_digest(),
        )
    };
    assert_eq!(
        ordinary.iter().map(attribution).collect::<Vec<_>>(),
        paint.iter().map(attribution).collect::<Vec<_>>()
    );
    let appearance = frame.appearance_text_candidates(instance).unwrap();
    assert_eq!(
        ordinary.iter().map(attribution).collect::<Vec<_>>(),
        appearance
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .map(attribution)
            .collect::<Vec<_>>()
    );
    let visual = frame
        .visual_region_basis()
        .for_binding(surface.binding, frame.receipt_basis.clone())
        .unsupported_paint();
    let visual = visual
        .iter()
        .filter(|row| row.node_receipt().mounted_instance() == instance)
        .collect::<Vec<_>>();
    assert_eq!(ordinary.len(), visual.len());
    for row in &ordinary {
        assert!(visual
            .iter()
            .any(|visual| visual.node_receipt() == row.node_receipt()
                && visual.bounds() == row.bounds()
                && visual.clip() == row.clip_bounds()
                && visual.source_digest() == row.semantic_digest()));
    }
    if let Some([width, height]) = expected {
        assert_eq!(ordinary.len(), 2);
        for row in ordinary {
            assert_eq!(row.clip_bounds().width(), width);
            assert_eq!(row.clip_bounds().height(), height);
        }
    } else {
        assert!(
            ordinary.is_empty(),
            "an invisible text row must not reach presentation"
        );
    }
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
    assert_eq!(context(&visible, child).text_foreground_spans.len(), 1);
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
    assert_eq!(context(&visible, child).text_foreground_spans.len(), 1);
}

#[test]
fn retained_value_cannot_substitute_for_missing_required_posture() {
    use worth_ui_host_contract::UiSemanticTextSlot;
    let mut world = GeometryWorld::new();
    let child = world.children[0];
    adopt_value(&mut world, child);
    let visible = world.frame(&[0, 1], None);
    let mut slots: Vec<_> = visible
        .appearance_text_candidates(child)
        .unwrap()
        .iter()
        .map(|row| row.slot())
        .collect();
    slots.sort();
    let mut expected = vec![UiSemanticTextSlot::Value, UiSemanticTextSlot::Posture];
    expected.sort();
    assert_eq!(slots, expected);
    let mut missing = visible.clone();
    missing.mechanics.remove_text_posture_for_test(child);
    let remaining = missing.appearance_text_candidates(child).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].slot(), UiSemanticTextSlot::Value);
    assert_eq!(
        missing
            .appearance_visible_foreground_spans(child)
            .unwrap_err(),
        crate::mounting::projection::UiMountedAppearanceOutputDenial::TextCandidate(
            crate::mounting::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
        ),
    );
    assert_eq!(context(&visible, child).text_foreground_spans.len(), 1);
}
