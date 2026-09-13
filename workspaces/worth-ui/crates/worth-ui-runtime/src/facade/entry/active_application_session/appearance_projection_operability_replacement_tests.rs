use super::*;

#[test]
fn source_replacement_clears_pending_operability_before_first_successor_paint() {
    replacement(false);
}

#[test]
fn rejected_pending_operability_replacement_preserves_paint_and_owner_then_retries() {
    replacement(true);
}

fn replacement(reject_first: bool) {
    let role = fixture::role();
    let successor = fixture::replacement_role(&role);
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "pending-operability-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 1);
    session
        .update_intent_boolean_fact(&fixture::fact("test.appearance.ready"), false)
        .unwrap();
    let (instance, decision) = activate(&mut session, surface, 1);
    assert_eq!(
        decision.primary_cause(),
        Some(UiIntentInoperableCause::Pending)
    );
    close(&mut session, &role, 1, "pending-operability-observed");
    let frame = project(
        &mut session,
        &[(instance, 30)],
        &[(
            surface,
            Some((instance, UiPointerAffordanceFamily::Default)),
        )],
    );
    publish(&mut session, &host, frame, 2);
    let predecessor = session.active_generation_identity();
    let before = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    let paint = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let source = fixture::candidate(&session, &successor, 1, "pending-operability-role-edit");
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        session.classify_observations(observations).unwrap()
    else {
        panic!("source attachment edit must change meaning");
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    if reject_first {
        host.push_rejected();
    } else {
        host.push_native_display_settled_without_effects();
    }
    let retry = match session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(3),
        )
        .unwrap()
        .execute(3)
    {
        crate::runtime::rebind::UiRebindOutcome::Published(_) if !reject_first => None,
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) if reject_first => {
            assert_eq!(
                denial.cause(),
                crate::runtime::rebind::UiRebindDenialCause::HostRejectedBeforeEffects
            );
            Some(
                denial
                    .detach_retry_for_native()
                    .unwrap_or_else(|_| panic!("retain prepared retry")),
            )
        }
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => {
            panic!("stateful replacement denied: {:?}", denial.cause())
        }
        _ => panic!("source replacement must reach its scripted outcome"),
    };
    if let Some(retry) = retry {
        assert_eq!(session.active_generation_identity(), predecessor);
        assert_eq!(
            session
                .intent_admission
                .operability_standing_snapshot()
                .unwrap(),
            before
        );
        assert_eq!(
            session.mounted.current_unpublished_appearance().unwrap(),
            paint.as_ref()
        );
        host.push_native_display_settled_without_effects();
        assert!(matches!(
            retry.rebase_content_and_retry(&mut session, 4).unwrap(),
            crate::runtime::rebind::UiRebindOutcome::Published(_)
        ));
    }
    assert_ne!(session.active_generation_identity(), predecessor);
    assert!(
        session
            .mounted
            .current_mounted_identity_basis(instance)
            .is_some(),
        "the same mounted owner and product route survive the attachment edit"
    );
    assert_eq!(
        session
            .application
            .prepared_authority()
            .intent_catalog()
            .single_product_route_identity(graph)
            .unwrap()
            .as_str(),
        fixture::ROUTE
    );
    assert!(session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap()
        .facts()
        .is_empty());
    assert_eq!(
        before
            .fact_for(graph, instance, fixture::ROUTE)
            .unwrap()
            .decision()
            .primary_cause(),
        Some(UiIntentInoperableCause::Pending)
    );
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let target = output.fragments().iter().find(|fragment| matches!(fragment.identity(),
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { predecessor: Some(receipt), successor: None }
        if receipt.mounted_instance() == instance)).unwrap_or_else(|| panic!("successor must remove Pending paint for {instance:?}: {output:?}"));
    // Cutover clears the standing evaluation. With no owner-issued decision,
    // the stateful role is unavailable; neither old Pending (red=30) nor a
    // fabricated Ready (red=10) may survive in the first successor pixels.
    assert!(target.work().successor().mechanics().is_empty());
    assert!(!target.work().changes().is_empty());
    assert!(target
        .work()
        .changes()
        .iter()
        .all(|change| matches!(change, UiMountedAppearanceMechanicChange::Remove(_))));
    let (successor_instance, fresh_decision) = activate(&mut session, surface, 3);
    assert_eq!(successor_instance, instance);
    assert_eq!(
        fresh_decision.primary_cause(),
        None,
        "candidate application facts reset readiness; only fresh owner evaluation can issue Ready"
    );
    let standing = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    let fact = standing.fact_for(graph, instance, fixture::ROUTE).unwrap();
    assert_eq!(fact.decision(), &fresh_decision);
    close(&mut session, &successor, 1, "successor-operability-issued");
    let frame = project(
        &mut session,
        &[(instance, 10)],
        &[(
            surface,
            Some((instance, UiPointerAffordanceFamily::Activation)),
        )],
    );
    publish(&mut session, &host, frame, 5);
    let _ = session.shutdown();
}
