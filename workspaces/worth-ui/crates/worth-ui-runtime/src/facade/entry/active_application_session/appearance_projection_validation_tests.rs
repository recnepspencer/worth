use crate::runtime::intent::UiValidationAppearanceClass as Class;
use worth_ui_dsl::*;
use worth_ui_host_contract::*;

#[test]
fn validation_changes_preserve_mounted_neighborhoods_and_unconsumed_snapshots() {
    let role = role();
    let (mut session, host) = super::theme_session(&role);
    let (_, graph_node) = super::mounting_fixture::mount(&mut session, 1_000);
    let node = session.mounted_graph_node(graph_node).unwrap();
    for _ in 0..2 {
        let surface = session.create_semantic_surface().unwrap();
        session
            .register_host_surface(
                surface,
                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                crate::facade::mounted::UiSurfaceBindingProfile::new(
                    1_000,
                    crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        session.mount_instance(node, surface).unwrap();
        crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
            &mut session,
            surface,
        );
    }
    close(&mut session, &role, "validation-initial");
    session.advance_mounted_identity_frame().unwrap();
    let instances = session.mounted_instances_for(node).unwrap();
    assert_eq!(instances.len(), 3);
    let targets = [instances[0], instances[1], instances[2]];
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial validation frame must prepare"));
    let initial_cost = frame.appearance_selection_cost_report();
    assert_eq!(initial_cost.selected_instance_count(), 3);
    assert_eq!(initial_cost.materialized_context_count(), 3);
    assert_eq!(initial_cost.index_entries_touched(), 4); // One graph index entry plus three mounts.
    assert_output(&frame, &targets.map(|target| (target, 10)));
    publish(&mut session, &host, frame, 1);

    set_class(&mut session, graph_node, targets[0], None, Class::Invalid);
    let first = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    close(&mut session, &role, "validation-invalid");
    let queued = session
        .presentation
        .appearance_invalidation_batch()
        .unwrap()
        .revision();

    // Repeating an identical admitted fact must preserve its revision and root.
    set_class(
        &mut session,
        graph_node,
        targets[0],
        Some(1),
        Class::Invalid,
    );
    let repeated = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert_eq!(first, repeated);
    assert!(repeated.changed_instances(&first).is_empty());
    close(&mut session, &role, "validation-equal");
    assert_eq!(
        session
            .presentation
            .appearance_invalidation_batch()
            .unwrap()
            .revision(),
        queued
    );
    let frame = project(&mut session, &[(targets[0], 30)]);
    publish(&mut session, &host, frame, 2);
    close(&mut session, &role, "validation-no-change");
    drop(project(&mut session, &[]));

    set_class(
        &mut session,
        graph_node,
        targets[0],
        Some(1),
        Class::Advisory,
    );
    let second = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert_eq!(first.fact_basis_for(graph_node, targets[0]).unwrap().1, 1);
    assert_eq!(second.fact_basis_for(graph_node, targets[0]).unwrap().1, 2);
    assert_eq!(second.changed_instances(&first).as_ref(), &[targets[0]]);

    // A close that is sealed but never classified must not consume the delta.
    let abandoned = seal(&mut session, &role, "validation-abandoned-close");
    drop(abandoned);
    set_class(&mut session, graph_node, targets[1], None, Class::Pending);
    close(&mut session, &role, "validation-after-abandoned-close");
    let frame = project(&mut session, &[(targets[0], 20), (targets[1], 40)]);
    publish(&mut session, &host, frame, 3);

    // Exercise the production owner retirement operation while retaining the
    // mount, so its removed fact must project Unspecified at that exact target.
    session
        .intent_application_facts
        .retire_validation_appearance_instance(targets[0]);
    close(&mut session, &role, "validation-retired-fact");
    let frame = project(&mut session, &[(targets[0], 10)]);
    publish(&mut session, &host, frame, 4);
    assert!(first.fact_basis_for(graph_node, targets[0]).is_some());
    let retired = session
        .intent_application_facts
        .validation_appearance_snapshot()
        .unwrap();
    assert!(retired.fact_basis_for(graph_node, targets[0]).is_none());
    assert!(retired.fact_basis_for(graph_node, targets[1]).is_some());
    close(&mut session, &role, "validation-retired-no-change");
    drop(project(&mut session, &[]));
    let _ = session.shutdown();
}

fn set_class(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    graph: crate::graph::UiGraphNodeIdentity,
    instance: UiMountedInstanceIdentity,
    expected: Option<u64>,
    class: Class,
) {
    let receipt = session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .find(|row| row.mounted_instance_identity() == instance)
        .unwrap()
        .node_receipt_identity();
    let admitted = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        session, graph, instance, receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(admitted, expected, class)
        .unwrap();
}

fn role() -> UiAppearanceRoleDeclaration {
    let mut table = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Validation,
    )]);
    for (class, red) in [
        (UiAppearanceAxisClass::ValidationUnspecified, 10),
        (UiAppearanceAxisClass::ValidationValid, 15),
        (UiAppearanceAxisClass::ValidationAdvisory, 20),
        (UiAppearanceAxisClass::ValidationInvalid, 30),
        (UiAppearanceAxisClass::ValidationPending, 40),
        (UiAppearanceAxisClass::ValidationStale, 50),
    ] {
        table = table.with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(class)]).literal(
                UiThemeValue::Color(UiThemeColor::from_channels([red, 0, 0, 255])),
            ),
        );
    }
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.validation-background").unwrap(),
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

fn seal(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    name: &str,
) -> crate::facade::observation::UiAdmittedObservationSet {
    let module = WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_appearance_role(role.clone())
        .with_component_appearance_role(
            super::support::APPEARANCE_NODE_A,
            UiAppearanceRoleAttachmentDeclaration::new(role.role().clone(), role.revision()),
        )
        .unwrap();
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored(name)
                .with_rust_authored_input(WorthUiRustAuthoredArtifactInput::from_modules([module])),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    turn.seal().unwrap()
}

fn close(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    name: &str,
) {
    let admitted = seal(session, role, name);
    session.classify_observations(admitted).unwrap();
}

fn project(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    expected: &[(UiMountedInstanceIdentity, u8)],
) -> crate::mounting::UiPreparedMountedFrame {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("validation appearance frame must prepare"));
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(cost.selected_instance_count(), expected.len());
    assert_eq!(cost.materialized_context_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    if expected.is_empty() {
        frame.assert_no_unpublished_appearance_for_test();
        return frame;
    }
    assert_output(&frame, expected);
    frame
}

fn assert_output(
    frame: &crate::mounting::UiPreparedMountedFrame,
    expected: &[(UiMountedInstanceIdentity, u8)],
) {
    let output = frame.lower_unpublished_appearance_for_test();
    let mut observed = Vec::new();
    for fragment in output.fragments() {
        let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } = fragment.identity()
        else {
            panic!("validation output requires a successor receipt");
        };
        assert_eq!(fragment.work().successor().mechanics().len(), 1);
        let UiMountedAppearanceMechanic::Surface(mechanic) =
            &fragment.work().successor().mechanics()[0]
        else {
            panic!("validation background requires a surface mechanic");
        };
        let red = expected
            .iter()
            .find(|(instance, _)| *instance == receipt.mounted_instance())
            .expect("unrelated target received validation appearance work")
            .1;
        assert_eq!(
            mechanic.paint(),
            &UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                red, 0, 0, 255
            ]))
        );
        observed.push(receipt.mounted_instance());
    }
    observed.sort_unstable();
    let mut identities = expected
        .iter()
        .map(|(instance, _)| *instance)
        .collect::<Vec<_>>();
    identities.sort_unstable();
    assert_eq!(observed, identities);
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    now: u64,
) {
    for (index, _) in frame.surfaces().iter().enumerate() {
        if now == 1 {
            // theme_session already scripts the first initial surface.
            if index != 0 {
                host.push_native_display_presented();
            }
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    use crate::mounting::UiMountedFrameOutcome as Outcome;
    match session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        now,
    ) {
        Outcome::Published(_) | Outcome::Unchanged(_) => {}
        Outcome::AdmissionDenied(rejection) => panic!("turn {now}: {:?}", rejection.denial()),
        Outcome::CompletionDenied(denial) => panic!("turn {now}: {denial:?}"),
        Outcome::RejectedBeforeEffects(rejection) => {
            panic!("turn {now}: {:?}", rejection.rejections())
        }
        Outcome::RetentionDenied(rejection) => panic!("turn {now}: {:?}", rejection.denial()),
        Outcome::PresentationIndeterminate(report) => panic!("turn {now}: {:?}", report.report()),
        Outcome::InFlight(pending) => panic!("turn {now}: {pending:?}"),
        Outcome::Reconciled(_) => panic!("turn {now}: unexpectedly reconciled"),
        Outcome::Superseded(_) => panic!("turn {now}: unexpectedly superseded"),
    }
}
