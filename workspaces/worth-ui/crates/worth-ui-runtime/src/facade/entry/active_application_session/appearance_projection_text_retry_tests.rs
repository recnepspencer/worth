use super::*;

#[test]
fn intent_consequence_publishes_pending_product_text_and_commits_only_after_acceptance() {
    product_text_retry(false);
}

#[test]
fn detached_content_retry_rebases_pending_product_text_and_commits_only_after_acceptance() {
    product_text_retry(true);
}

fn product_text_retry(detached: bool) {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let _ = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "intent-product-copy-initial");
    let initial = prepare(&mut session);
    let surface = initial.manifest().surfaces()[0].semantic_surface();
    fixture::publish(&mut session, &host, initial, 1);
    assert_accepted_text(&host, surface, "AB");
    let predecessor_text = host.accepted_text_commands(surface).unwrap();
    let predecessor = session.mounted.current_publication().unwrap().frame();

    set_text(&mut session, 1, "CD");
    if detached {
        let prepared = Box::new(
            crate::facade::entry::WorthUiPreparedMountedContentRebind::prepare(
                &mut session,
                crate::mounting::UiMountedSemanticContentInput::empty(),
            )
            .unwrap(),
        );
        assert_eq!(
            text_candidate_named(prepared.frame(), adopted, 10, "CD").text(),
            "CD"
        );
        host.push_rejected();
        let crate::facade::entry::WorthUiMountedContentRebindOutcome::RejectedBeforeEffects {
            retry,
            ..
        } = prepared.present(UiPresentationDeadline::at_tick(100), 2)
        else {
            panic!("content publication must retain its prepared retry");
        };
        let parked = retry.detach();
        assert_eq!(
            session.mounted.current_publication().unwrap().frame(),
            predecessor
        );
        assert_eq!(
            host.accepted_text_commands(surface).unwrap(),
            predecessor_text
        );
        // A parked retry must capture the current owner revision when rebased.
        set_text(&mut session, 2, "EF");
        let rebased = parked
            .rebase(
                &mut session,
                crate::mounting::UiMountedSemanticContentInput::empty(),
            )
            .unwrap();
        assert_eq!(
            text_candidate_named(rebased.frame(), adopted, 10, "EF").text(),
            "EF"
        );
        host.push_native_display_presented();
        let crate::facade::entry::WorthUiMountedContentRebindOutcome::Published(receipt) =
            rebased.present(UiPresentationDeadline::at_tick(100), 3)
        else {
            panic!("rebased content reaches acceptance");
        };
        assert_eq!(
            host.accepted_text_commands(surface).unwrap().0,
            receipt.into_parts().0.attempt()
        );
    } else {
        let successor = session
            .prepare_intent_consequence_frame(
                crate::mounting::UiMountedSemanticContentInput::empty(),
                0,
                Vec::new(),
            )
            .unwrap();
        assert_eq!(
            text_candidate_named(&successor, adopted, 10, "CD").text(),
            "CD"
        );
        host.push_rejected();
        let crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) = session
            .present_prepared_mounted_frame_internal(
                successor,
                UiPresentationDeadline::at_tick(100),
                2,
            )
        else {
            panic!("product feedback must remain pending after host rejection");
        };
        assert_eq!(
            session.mounted.current_publication().unwrap().frame(),
            predecessor
        );
        let retry = rejected.into_frame();
        assert_eq!(
            host.accepted_text_commands(surface).unwrap(),
            predecessor_text
        );
        assert_eq!(text_candidate_named(&retry, adopted, 10, "CD").text(), "CD");
        fixture::publish(&mut session, &host, retry, 2);
    }

    assert_accepted_text(&host, surface, if detached { "EF" } else { "CD" });
    // Consuming the exact pending text revision is observable: the next
    // ordinary preparation must not re-emit the accepted foreground update.
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
    assert_eq!(host.pending_presentation_count(), 0);
}

fn assert_accepted_text(
    host: &crate::certification_support::ScriptedPresentationHost,
    surface: UiSemanticSurfaceIdentity,
    expected: &str,
) {
    let (_, commands) = host.accepted_text_commands(surface).unwrap();
    let values = commands
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::SemanticText { mechanic, .. }
                if mechanic.slot() == UiSemanticTextSlot::Value =>
            {
                Some(mechanic.text())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [expected],
        "the first accepted host frame consumes the current product text"
    );
}

#[test]
fn text_content_successor_refreshes_foreground_coverage_without_re_resolving_ui_state() {
    let role = fixture::foreground_role();
    let contract = text_contract();
    let adopted = contract.scalar_spans()[0].paint_identity();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(contract));
    let _ = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "foreground-content-successor-initial");
    let initial = prepare(&mut session);
    let original = text_candidate(&initial, adopted, 10);
    fixture::publish(&mut session, &host, initial, 1);

    // This content-only successor keeps the role, theme, command identity,
    // placement, and span identity. Its new content generation must still
    // replace native foreground coverage for the current glyphs.
    set_text(&mut session, 1, "CD");
    let successor = prepare(&mut session);
    assert_text_damage_transition(&successor, UiAppearanceTextDamageTransition::Replace);
    let current = text_candidate_named(&successor, adopted, 10, "CD");
    assert_eq!(current.text(), "CD");
    assert_eq!(current.mounted_instance(), original.mounted_instance());
    assert_eq!(current.foregrounds(), original.foregrounds());
    assert_eq!(current.bounds(), original.bounds());
    host.push_rejected();
    let crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) = session
        .present_prepared_mounted_frame_internal(
            successor,
            UiPresentationDeadline::at_tick(100),
            2,
        )
    else {
        panic!("the host must reject this attempt before effects");
    };
    // Native atlas deferral retries this exact frame; rebuilding from the
    // session would conceal consumption of the pending foreground refresh.
    let successor = rejected.into_frame();
    assert_text_damage_transition(&successor, UiAppearanceTextDamageTransition::Replace);
    assert_eq!(
        text_candidate_named(&successor, adopted, 10, "CD").text(),
        "CD"
    );
    fixture::publish(&mut session, &host, successor, 2);

    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}
