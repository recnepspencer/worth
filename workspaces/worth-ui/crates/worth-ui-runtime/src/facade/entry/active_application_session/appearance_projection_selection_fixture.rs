#[path = "appearance_projection_selection_intent_fixture.rs"]
mod intent;
use super::super::support;
use worth_ui_dsl::*;
use worth_ui_host_contract::*;

pub(super) const PROJECTION: &str = "test.appearance.selection";

pub(super) fn role() -> UiAppearanceRoleDeclaration {
    let mut table = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Selection,
    )]);
    for (class, red) in [
        (UiAppearanceAxisClass::SelectionUnselected, 10),
        (UiAppearanceAxisClass::SelectionSelected, 20),
        (UiAppearanceAxisClass::SelectionAnchor, 30),
        (UiAppearanceAxisClass::SelectionCursor, 40),
        (UiAppearanceAxisClass::SelectedAnchorCursor, 50),
    ] {
        table = table.with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(class)]).literal(
                UiThemeValue::Color(UiThemeColor::from_channels([red, 0, 0, 255])),
            ),
        );
    }
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.selection-background").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::AnyComponent,
        &UiAppearanceAspectContract::component([UiAppearanceAspect::Background], []).unwrap(),
        [(
            UiAppearanceAspect::Background,
            table.compile(UiAppearanceAspect::Background).unwrap(),
        )],
    )
    .unwrap()
}

pub(super) fn session(
    registration: worth_ui_query_binding::UiCollectionProjectionRegistration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let role = role();
    let module = module();
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let observer = host.clone();
    let session = intent::register(support::legacy_static_paint_appearance_component_builder(
        &role,
    ))
    .register_appearance_theme_bundle(super::super::test_support::theme_bundle())
    .unwrap()
    .register_collection_projection(registration)
    .unwrap()
    .with_selection_policy_defaults(crate::declaration::UiSelectionPolicy::multiple())
    .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module]))
    .freeze()
    .map(|application| {
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            application,
            host,
        )
    })
    .unwrap()
    .launch()
    .unwrap();
    (session, observer)
}

fn module() -> WorthUiRustAuthoredArtifactInputModule {
    let role = role();
    WorthUiRustAuthoredArtifactInputModule::new("appearance/selection")
        .with_control_routes(
            support::APPEARANCE_NODE_A,
            [WorthUiIntentInteractionRoute::product(
                WorthUiIntentInteractionFamily::SelectionCommit,
                "test.mounted_selection.selection_declaration",
            )],
        )
        .with_intent_declaration(intent::declaration())
        .with_appearance_role(role.clone())
        .with_component_appearance_role(
            support::APPEARANCE_NODE_B,
            UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
        )
        .unwrap()
}

pub(super) fn publish_query(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    observation: worth_ui_query_binding::UiProjectionObservation,
) {
    for _ in session.inspect_mounted_identity().surface_bindings() {
        host.push_native_display_settled_without_effects();
    }
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(observation).unwrap();
    let admitted = turn.seal().unwrap();
    let crate::facade::observation::UiChangeClassificationOutcome::Changed(changed) =
        session.classify_observations(admitted).unwrap()
    else {
        panic!("collection changes")
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
    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(71),
        )
        .unwrap();
    match prepared.execute(71) {
        crate::runtime::rebind::UiRebindOutcome::Published(_) => {}
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => {
            panic!("collection rebind: {:?}", denial.cause())
        }
        crate::runtime::rebind::UiRebindOutcome::InternalDefect(defect) => {
            panic!("collection rebind defect: {:?}", defect.kind())
        }
        crate::runtime::rebind::UiRebindOutcome::Indeterminate(recovery) => panic!(
            "collection rebind indeterminate: {:?}",
            recovery.frame().report()
        ),
        crate::runtime::rebind::UiRebindOutcome::InFlight(_) => {
            panic!("collection rebind in flight")
        }
        crate::runtime::rebind::UiRebindOutcome::Duplicate(_) => {
            panic!("collection rebind duplicate")
        }
        crate::runtime::rebind::UiRebindOutcome::ObservedNoChange(_) => {
            panic!("collection rebind unchanged")
        }
        crate::runtime::rebind::UiRebindOutcome::CancelledBeforeEffects(_) => {
            panic!("collection rebind cancelled")
        }
        crate::runtime::rebind::UiRebindOutcome::TimedOutBeforeEffects(_) => {
            panic!("collection rebind timed out")
        }
        crate::runtime::rebind::UiRebindOutcome::SupersededBeforeEffects(_) => {
            panic!("collection rebind superseded")
        }
    }
}

pub(super) fn close(session: &mut crate::facade::WorthUiActiveApplicationSession, name: &str) {
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored(name).with_rust_authored_input(
                WorthUiRustAuthoredArtifactInput::from_modules([module()]),
            ),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
}

pub(super) fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    now: u64,
) {
    for _ in frame.surfaces() {
        if now == 1 {
            host.push_native_display_presented();
        } else {
            // This fixture changes only unpublished appearance and Query facts.
            host.push_native_display_settled_without_effects();
        }
    }
    let result = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(1000),
        now,
    );
    let diagnostic = match &result {
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => {
            format!("admission: {:?}", denial.denial())
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            format!("rejected: {:?}", rejected.rejections())
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
            format!("indeterminate: {:?}", indeterminate.report())
        }
        other => format!("outcome: {:?}", std::mem::discriminant(other)),
    };
    assert!(
        matches!(
            result,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ),
        "mounted Selection frame {now} settles: {diagnostic}"
    );
}

pub(super) fn frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
) -> crate::mounting::UiPreparedMountedFrame {
    session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("Selection frame prepares"))
}

pub(super) fn receipt(
    session: &crate::facade::WorthUiActiveApplicationSession,
    item: UiMountedInstanceIdentity,
) -> UiMountedNodeReceiptIdentity {
    session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .find(|row| row.mounted_instance_identity() == item)
        .expect("current mounted receipt")
        .node_receipt_identity()
}

pub(super) fn assert_output(
    frame: &crate::mounting::UiPreparedMountedFrame,
    expected: &[(UiMountedInstanceIdentity, u8)],
) {
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(cost.selected_instance_count(), expected.len());
    assert_eq!(cost.materialized_context_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    if expected.is_empty() {
        frame.assert_no_unpublished_appearance_for_test();
        return;
    }
    let output = frame.lower_unpublished_appearance_for_test();
    let mut observed = Vec::new();
    for fragment in output.fragments() {
        let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } = fragment.identity()
        else {
            panic!("item output has receipt")
        };
        let red = expected
            .iter()
            .find(|(item, _)| *item == receipt.mounted_instance())
            .expect("unrelated item received work")
            .1;
        let [UiMountedAppearanceMechanic::Surface(mechanic)] =
            fragment.work().successor().mechanics()
        else {
            panic!("one background")
        };
        assert_eq!(
            mechanic.paint(),
            &UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                red, 0, 0, 255
            ]))
        );
        observed.push(receipt.mounted_instance());
    }
    observed.sort_unstable();
    let mut expected = expected.iter().map(|(item, _)| *item).collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(observed, expected);
}
