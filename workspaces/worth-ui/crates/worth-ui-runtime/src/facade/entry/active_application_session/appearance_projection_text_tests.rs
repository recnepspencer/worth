use super::fixture;
use worth_ui_host_contract::*;

#[path = "appearance_projection_text_geometry_tests.rs"]
mod geometry_tests;
#[path = "appearance_projection_text_publication_tests.rs"]
mod publication_tests;

#[test]
fn foreground_adoption_changes_contract_meaning_without_reidentifying_spans() {
    let adopted = text_contract();
    let retained = crate::capability::ComponentSemanticTextContract::spanned(
        adopted.theme_token().clone(),
        adopted.layer_semantic_order(),
        adopted.scalar_spans().iter().map(|span| {
            crate::capability::ComponentSemanticTextSpanContract::new(
                span.original_range(),
                span.foreground_token().clone(),
                span.style().clone(),
            )
            .unwrap()
        }),
    )
    .unwrap();
    assert_ne!(adopted.digest_basis(), retained.digest_basis());
    for (adopted, retained) in adopted.scalar_spans().iter().zip(retained.scalar_spans()) {
        assert_eq!(adopted.paint_identity(), retained.paint_identity());
        assert_eq!(adopted.original_range(), retained.original_range());
        assert_eq!(adopted.style(), retained.style());
    }
}

#[test]
fn authored_foreground_changes_only_adopted_original_ranges() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    session
        .admit_application_semantic_text(&[
            crate::native_platform::UiNativeComponentSemanticTextChange::new(
                format!("component:{}", super::super::support::APPEARANCE_NODE_A),
                "AB",
            )
            .unwrap(),
        ])
        .unwrap();
    fixture::close_source(&mut session, &role, "foreground-initial");
    let initial = prepare(&mut session);
    let original = text_candidate(&initial, adopted, 10);
    assert_text_damage_transition(&initial, UiAppearanceTextDamageTransition::Insert);
    assert_eq!(original.text(), "AB");
    assert_eq!(original.foregrounds().len(), 2);
    assert_eq!(
        original.foregrounds()[0].original_range(),
        UiTextOriginalRange::new(0, 1).unwrap()
    );
    assert_eq!(
        original.foregrounds()[1].original_range(),
        UiTextOriginalRange::new(1, 2).unwrap()
    );
    fixture::publish(&mut session, &host, initial, 1);

    super::observe(&mut session, surface, true);
    let first = super::traverse(&mut session, surface);
    let focused = if first == Some(original.mounted_instance()) {
        first
    } else {
        super::traverse(&mut session, surface)
    };
    assert_eq!(focused, Some(original.mounted_instance()));
    fixture::close_source(&mut session, &role, "foreground-focused");
    let changed = prepare(&mut session);
    let current = text_candidate(&changed, adopted, 30);
    assert_text_damage_transition(&changed, UiAppearanceTextDamageTransition::Replace);
    assert_eq!(
        current.qualified_layout_identity(),
        original.qualified_layout_identity()
    );
    assert_eq!(
        current.qualified_layout_request(),
        original.qualified_layout_request()
    );
    assert_eq!(current.foregrounds(), original.foregrounds());
    assert_eq!(current.bounds(), original.bounds());
    assert_eq!(current.clip_bounds(), original.clip_bounds());
    assert_eq!(current.performed_layout_cost(), None);
    fixture::publish(&mut session, &host, changed, 2);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}

#[test]
fn text_content_alone_removes_and_restores_adopted_foreground() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let _ = super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "foreground-content-initial");
    let initial = prepare(&mut session);
    let original = text_candidate(&initial, adopted, 10);
    fixture::publish(&mut session, &host, initial, 1);

    // No source close, theme mutation, or owner-state change supplies this work.
    set_text(&mut session, 1, "");
    let abandoned = prepare(&mut session);
    assert_foreground_removed(&abandoned);
    drop(abandoned);
    let cleared = prepare(&mut session);
    assert_foreground_removed(&cleared);
    let removed_output = cleared.lower_unpublished_appearance_for_test();
    let removed = removed_output.fragments()[0]
        .work()
        .text_damage_requirements()
        .next()
        .unwrap();
    assert_eq!(removed.target(), original.mounted_instance());
    assert_eq!(removed.span().digest(), adopted);
    fixture::publish(&mut session, &host, cleared, 2);

    set_text(&mut session, 2, "AB");
    let restored = prepare(&mut session);
    let current = text_candidate(&restored, adopted, 10);
    assert_eq!(current.mounted_instance(), original.mounted_instance());
    assert_eq!(current.foregrounds(), original.foregrounds());
    fixture::publish(&mut session, &host, restored, 3);

    let spans = text_contract()
        .scalar_spans()
        .iter()
        .map(|span| {
            crate::capability::ComponentSemanticTextSpanContract::new(
                span.original_range(),
                span.foreground_token().clone(),
                span.style().clone(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    set_spans(&mut session, 3, spans);
    let unadopted = prepare(&mut session);
    assert_foreground_removed(&unadopted);
    host.push_rejected();
    let rejected = session.present_prepared_mounted_frame_internal(
        unadopted,
        UiPresentationDeadline::at_tick(100),
        4,
    );
    assert!(matches!(
        rejected,
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    drop(rejected);
    let retried = prepare(&mut session);
    assert_foreground_removed(&retried);
    fixture::publish(&mut session, &host, retried, 5);
    set_spans(&mut session, 4, text_contract().scalar_spans().to_vec());
    let readopted = prepare(&mut session);
    let readopted_text = text_candidate(&readopted, adopted, 10);
    assert_eq!(
        readopted_text.qualified_layout_identity(),
        current.qualified_layout_identity()
    );
    assert_eq!(readopted_text.performed_layout_cost(), None);
    fixture::publish(&mut session, &host, readopted, 6);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}

fn set_text(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    revision: u64,
    text: &str,
) {
    session
        .admit_application_semantic_text(&[
            crate::native_platform::UiNativeComponentSemanticTextChange::successor(
                format!("component:{}", super::super::support::APPEARANCE_NODE_A),
                revision,
                text,
            )
            .unwrap(),
        ])
        .unwrap();
}

fn set_spans(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    revision: u64,
    spans: Vec<crate::capability::ComponentSemanticTextSpanContract>,
) {
    session
        .admit_application_semantic_text(&[
            crate::native_platform::UiNativeComponentSemanticTextChange::successor(
                format!("component:{}", super::super::support::APPEARANCE_NODE_A),
                revision,
                "AB",
            )
            .unwrap()
            .with_spans(spans)
            .unwrap(),
        ])
        .unwrap();
}

fn assert_foreground_removed(frame: &crate::mounting::UiPreparedMountedFrame) {
    assert_text_damage_transition(frame, UiAppearanceTextDamageTransition::Remove);
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), 1);
    let fragment = &output.fragments()[0];
    assert!(fragment.work().successor().mechanics().is_empty());
    assert_eq!(fragment.work().changes().len(), 1);
    assert!(matches!(
        &fragment.work().changes()[0],
        UiMountedAppearanceMechanicChange::Remove(
            UiMountedAppearanceMechanicIdentity::TextForeground { .. }
        )
    ));
    assert!(fragment.text_candidates().is_empty());
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output).unwrap();
}

fn text_candidate(
    frame: &crate::mounting::UiPreparedMountedFrame,
    adopted: [u8; 32],
    red: u8,
) -> UiMountedSemanticTextMechanic {
    let output = frame.lower_unpublished_appearance_for_test();
    let texts = output
        .fragments()
        .iter()
        .flat_map(|fragment| {
            fragment
                .work()
                .successor()
                .mechanics()
                .iter()
                .filter_map(move |mechanic| {
                    if let UiMountedAppearanceMechanic::TextForeground(text) = mechanic {
                        Some((fragment, text))
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 1);
    let (fragment, foreground) = texts[0];
    let requirements = fragment
        .work()
        .text_damage_requirements()
        .collect::<Vec<_>>();
    assert_eq!(requirements.len(), 1);
    assert_eq!(
        requirements[0].target(),
        foreground.node_receipt().mounted_instance()
    );
    assert_eq!(requirements[0].span(), foreground.paint_span());
    assert!(
        fragment.work().damage().is_empty(),
        "allocation is not completed text damage"
    );
    assert_eq!(foreground.paint_span().digest(), adopted);
    assert_eq!(
        foreground.foreground(),
        UiMountedAppearanceColor::from_straight_srgba([red, 0, 0, 255])
    );
    assert_eq!(fragment.text_candidates().len(), 2);
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output).unwrap();
    fragment
        .text_candidates()
        .iter()
        .find(|row| row.text() == "AB")
        .unwrap()
        .clone()
}

fn assert_text_damage_transition(
    frame: &crate::mounting::UiPreparedMountedFrame,
    expected: UiAppearanceTextDamageTransition,
) {
    let output = frame.lower_unpublished_appearance_for_test();
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output)
            .unwrap();
    let requirements = transcript
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().text_damage_requirements())
        .collect::<Vec<_>>();
    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].transition(), expected);
    assert!(transcript
        .fragments()
        .iter()
        .filter(|fragment| !fragment.work().text_damage_requirements().is_empty())
        .all(|fragment| fragment.work().damage().is_empty()));
}

fn prepare(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
) -> crate::mounting::UiPreparedMountedFrame {
    session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("foreground frame prepares"))
}

fn text_contract() -> crate::capability::ComponentSemanticTextContract {
    use crate::capability::*;
    let token = ThemeTokenId::new(super::super::support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    let constraints = worth_ui_text::UiTextParagraphConstraints::new(
        worth_ui_text::UiTextParagraphConstraintsInput {
            language: std::sync::Arc::from("und"),
            base_direction: worth_ui_text::UiTextBaseDirection::Auto,
            wrap: worth_ui_text::UiTextWrap::UnicodeWord,
            alignment: worth_ui_text::UiTextAlignment::Start,
            overflow: worth_ui_text::UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 160_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 1,
        },
    )
    .unwrap();
    let span = |start, end| {
        ComponentSemanticTextSpanContract::new(
            UiTextOriginalRange::new(start, end).unwrap(),
            token.clone(),
            worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        )
        .unwrap()
    };
    ComponentSemanticTextContract::spanned(
        token.clone(),
        7,
        [span(0, 1).with_appearance_foreground(), span(1, 2)],
    )
    .unwrap()
}
