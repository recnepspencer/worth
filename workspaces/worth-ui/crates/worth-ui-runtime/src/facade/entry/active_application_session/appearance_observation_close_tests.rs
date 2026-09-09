use crate::runtime::tests::active_application_session_test_support::admit_candidate_catalog;
use crate::runtime::tests::appearance_component_session_test_support::source_backed_static_paint_consumer_session;
use crate::runtime::tests::appearance_component_session_test_support::{
    appearance_candidate_submission, attached_appearance_candidate_submission,
    source_backed_static_paint_role_capable_session, validation_background_role,
};

#[test]
fn sealed_turn_carries_pre_interleaving_owner_state_into_classification() {
    let mut session = source_backed_static_paint_consumer_session();
    let first_candidate = attached_appearance_candidate_submission(
        &session,
        "appearance-close-first",
        "workspace.component.active_session_current",
    );
    let mut first_turn = session.begin_observation_turn().unwrap();
    first_turn.admit_source(first_candidate).unwrap();
    let first = first_turn.seal().unwrap();
    let first_turn_identity = first.turn();
    assert_eq!(validation_revision(&first), 0);

    let _ = publish_invalid_validation_fact(&mut session);

    let second_candidate = attached_appearance_candidate_submission(
        &session,
        "appearance-close-second",
        "workspace.component.active_session_candidate",
    );
    let mut second_turn = session.begin_observation_turn().unwrap();
    second_turn.admit_source(second_candidate).unwrap();
    let second = second_turn.seal().unwrap();
    let second_turn_identity = second.turn();
    assert_eq!(validation_revision(&second), 1);

    session.classify_observations(first).unwrap();
    let retained = session.appearance_owner_snapshot_for_test().unwrap();
    assert_eq!(retained.turn(), first_turn_identity);
    assert_eq!(retained.validation().unwrap().owner_revision(), 0);

    session.classify_observations(second).unwrap();
    let retained = session.appearance_owner_snapshot_for_test().unwrap();
    assert_eq!(retained.turn(), second_turn_identity);
    assert_eq!(retained.validation().unwrap().owner_revision(), 1);
    let _ = session.shutdown();
}

#[test]
fn cutover_reconciles_validation_owner_only_while_the_axis_is_consumed() {
    let role = validation_background_role("theme.appearance_consumer");
    let mut session = source_backed_static_paint_role_capable_session(&role);
    assert!(!session.has_appearance_owner_snapshot_for_test());

    let enable = appearance_candidate_submission(&session, "appearance-enable", Some(&role));
    activate_candidate(&mut session, enable);
    let mounted = publish_invalid_validation_fact(&mut session);
    let enabled_observation =
        appearance_candidate_submission(&session, "appearance-enabled-observation", Some(&role));
    let mut enabled_turn = session.begin_observation_turn().unwrap();
    enabled_turn.admit_source(enabled_observation).unwrap();
    let enabled = enabled_turn.seal().unwrap();
    assert!(enabled.carries_appearance_owner_snapshot_for_test());
    assert_eq!(validation_revision(&enabled), 1);
    session.unmount_instance(mounted).unwrap();

    let disable = appearance_candidate_submission(&session, "appearance-disable", None);
    activate_candidate(&mut session, disable);
    assert!(!session.has_appearance_owner_snapshot_for_test());
    let disabled_observation =
        appearance_candidate_submission(&session, "appearance-disabled-observation", None);
    let mut disabled_turn = session.begin_observation_turn().unwrap();
    disabled_turn.admit_source(disabled_observation).unwrap();
    let disabled = disabled_turn.seal().unwrap();
    assert!(!disabled.carries_appearance_owner_snapshot_for_test());
    let _ = session.shutdown();
}

#[test]
fn validation_fact_receipt_succession_is_current_and_unmount_removes_the_row() {
    let mut session = source_backed_static_paint_consumer_session();
    let (graph_node, instance, first_receipt) = mount_appearance_consumer(&mut session);
    let first_target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        &session,
        graph_node,
        instance,
        first_receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(
            first_target,
            None,
            crate::runtime::intent::UiValidationAppearanceClass::Invalid,
        )
        .unwrap();
    let first = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    let (identity, revision, _) = first.fact_basis_for(graph_node, instance).unwrap();
    assert_eq!(revision, 1);

    session.advance_mounted_identity_frame().unwrap();
    let second_receipt = current_receipt(&session, instance);
    let second_target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        &session,
        graph_node,
        instance,
        second_receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(
            second_target,
            Some(revision),
            crate::runtime::intent::UiValidationAppearanceClass::Advisory,
        )
        .unwrap();
    let second = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert_eq!(second.fact_count(), 1);
    assert_eq!(
        second.fact_basis_for(graph_node, instance).unwrap().0,
        identity
    );
    assert_eq!(
        second.class_for(graph_node, instance, Some(first_receipt)),
        Some(crate::runtime::intent::UiValidationAppearanceClass::Stale)
    );
    assert_eq!(
        second.class_for(graph_node, instance, None),
        Some(crate::runtime::intent::UiValidationAppearanceClass::Stale)
    );
    assert_eq!(
        second.class_for(graph_node, instance, Some(second_receipt)),
        Some(crate::runtime::intent::UiValidationAppearanceClass::Advisory)
    );
    let stale_target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        &session,
        graph_node,
        instance,
        second_receipt,
    )
    .unwrap();
    assert_eq!(
        session
            .intent_application_facts
            .publish_validation_appearance_fact(
                stale_target,
                Some(revision),
                crate::runtime::intent::UiValidationAppearanceClass::Invalid,
            ),
        Err(crate::runtime::intent::UiValidationAppearanceFactDenial::StalePredecessor)
    );
    let preserved = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert_eq!(
        preserved.fact_basis_for(graph_node, instance),
        second.fact_basis_for(graph_node, instance)
    );
    assert_eq!(preserved.owner_revision(), second.owner_revision());

    session.unmount_instance(instance).unwrap();
    let retired = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert_eq!(retired.fact_count(), 0);
    assert_eq!(retired.owner_revision(), second.owner_revision() + 1);
    assert_eq!(
        retired.class_for(graph_node, instance, Some(second_receipt)),
        None
    );
    let _ = session.shutdown();
}

#[test]
fn rejected_foreign_turn_preserves_the_predecessor_owner_snapshot() {
    let mut first = source_backed_static_paint_consumer_session();
    let foreign_candidate = attached_appearance_candidate_submission(
        &first,
        "appearance-close-foreign",
        "workspace.component.active_session_current",
    );
    let mut foreign_turn = first.begin_observation_turn().unwrap();
    foreign_turn.admit_source(foreign_candidate).unwrap();
    let foreign = foreign_turn.seal().unwrap();

    let mut second = source_backed_static_paint_consumer_session();
    let local_candidate = attached_appearance_candidate_submission(
        &second,
        "appearance-close-local",
        "workspace.component.active_session_current",
    );
    let mut local_turn = second.begin_observation_turn().unwrap();
    local_turn.admit_source(local_candidate).unwrap();
    let local = local_turn.seal().unwrap();
    let local_turn_identity = local.turn();
    second.classify_observations(local).unwrap();

    assert!(matches!(
        second.classify_observations(foreign),
        Err(crate::runtime::observation::UiChangeClassificationDenial::ForeignSession)
    ));
    assert_eq!(
        second.appearance_owner_snapshot_for_test().unwrap().turn(),
        local_turn_identity,
    );
    let _ = first.shutdown();
    let _ = second.shutdown();
}

#[test]
fn theme_switch_origin_requires_exact_family_and_session() {
    use crate::runtime::appearance::{
        UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily,
    };

    let mut first = source_backed_static_paint_consumer_session();
    let candidate = attached_appearance_candidate_submission(
        &first,
        "appearance-origin-source",
        "workspace.component.active_session_current",
    );
    let mut turn = first.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    assert!(first
        .issue_theme_switch_origin(&admitted, UiThemeSwitchOriginFamily::SourceEditObservation,)
        .is_ok());
    assert_eq!(
        first.issue_theme_switch_origin(
            &admitted,
            UiThemeSwitchOriginFamily::ProgrammaticObservation,
        ),
        Err(UiThemeSwitchOriginAdmissionDenial::MissingRequiredObservationFamily)
    );

    let foreign = source_backed_static_paint_consumer_session();
    assert_eq!(
        foreign.issue_theme_switch_origin(
            &admitted,
            UiThemeSwitchOriginFamily::SourceEditObservation,
        ),
        Err(UiThemeSwitchOriginAdmissionDenial::ForeignSession)
    );
    let _ = first.shutdown();
    let _ = foreign.shutdown();
}

#[test]
fn sealed_turn_cannot_be_classified_after_application_cutover() {
    let mut session = source_backed_static_paint_consumer_session();
    let candidate = attached_appearance_candidate_submission(
        &session,
        "appearance-close-before-cutover",
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let sealed_before_cutover = turn.seal().unwrap();

    let mut prepared = session
        .prepare_replacement(attached_appearance_candidate_submission(
            &session,
            "appearance-close-successor",
            "workspace.component.active_session_candidate",
        ))
        .unwrap();
    let catalog = admit_candidate_catalog(&mut prepared);
    let lowered = session.lower_prepared_replacement(*prepared).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();
    let _ = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .unwrap();

    assert!(matches!(
        session.classify_observations(sealed_before_cutover),
        Err(
            crate::runtime::observation::UiChangeClassificationDenial::ForeignApplicationGeneration
        )
    ));
    let _ = session.shutdown();
}

fn validation_revision(set: &crate::runtime::observation::UiAdmittedObservationSet) -> u64 {
    set.appearance_owner_snapshot_for_test()
        .expect("appearance consumer turns carry a close snapshot")
        .validation()
        .expect("the admitted role consumes validation")
        .owner_revision()
}

fn publish_invalid_validation_fact(
    session: &mut super::WorthUiActiveApplicationSession,
) -> worth_ui_host_contract::UiMountedInstanceIdentity {
    let (graph_node, instance, receipt) = mount_appearance_consumer(session);
    let target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        session, graph_node, instance, receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(
            target,
            None,
            crate::runtime::intent::UiValidationAppearanceClass::Invalid,
        )
        .unwrap();
    instance
}

fn activate_candidate(
    session: &mut super::WorthUiActiveApplicationSession,
    candidate: crate::runtime::WorthUiWatchedCandidateSubmission,
) {
    let mut prepared = session.prepare_replacement(candidate).unwrap();
    let catalog = admit_candidate_catalog(&mut prepared);
    let lowered = session.lower_prepared_replacement(*prepared).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();
    let _ = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .unwrap();
}

fn mount_appearance_consumer(
    session: &mut super::WorthUiActiveApplicationSession,
) -> (
    crate::graph::UiGraphNodeIdentity,
    worth_ui_host_contract::UiMountedInstanceIdentity,
    worth_ui_host_contract::UiMountedNodeReceiptIdentity,
) {
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.appearance_role_attachment().is_some())
        .expect("appearance consumer graph has one attached node")
        .graph_node_identity();
    let node = session.mounted_graph_node(graph_node).unwrap();
    let surface = session.create_semantic_surface().unwrap();
    let instance = session.mount_instance(node, surface).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    (graph_node, instance, current_receipt(session, instance))
}

fn current_receipt(
    session: &super::WorthUiActiveApplicationSession,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
    session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .copied()
        .find(|row| row.mounted_instance_identity() == instance)
        .expect("the current frame carries the mounted appearance consumer")
        .node_receipt_identity()
}
