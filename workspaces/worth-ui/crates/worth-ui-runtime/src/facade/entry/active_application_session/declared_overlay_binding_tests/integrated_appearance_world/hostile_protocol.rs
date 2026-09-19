use crate::facade::observation_report::{
    UiHostObservationReportDenial, UiHostObservationReportOutcome,
};
use worth_ui_host_contract::*;

#[path = "hostile_protocol/selection_intent.rs"]
mod selection_intent;

pub(super) const SELECTION_PROJECTION: &str = "integrated.ap07.selection";

pub(super) fn selection_registration() -> worth_ui_query_binding::UiCollectionProjectionRegistration
{
    use worth_ui_query_binding::*;
    let (query, _) = certification::seeded_collection_projection_workspace_with_item_keys(
        vec![("selected".into(), "Selected".into(), 7)],
        certification::WorthUiCollectionProjectionSeedPosture::Complete,
    );
    let domain = query.worth_ui().unwrap();
    UiCollectionProjectionRegistration::text(
        domain.projection_view(SELECTION_PROJECTION).unwrap(),
        UiProjectionFieldRequirement::identity_id(),
        [UiProjectionFieldRequirement::query_text_status()],
        false,
        false,
    )
    .unwrap()
    .with_unsigned64_application_item_key_field(UiProjectionFieldRequirement::collection_item_key())
}

pub(super) fn configure_builder(
    builder: crate::facade::entry::WorthUiCertificationApplicationBuilder,
    registration: worth_ui_query_binding::UiCollectionProjectionRegistration,
) -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    selection_intent::register(builder)
        .register_appearance_role(super::authored::six_axis_role())
        .unwrap()
        .register_collection_projection(registration)
        .unwrap()
        .with_selection_policy_defaults(crate::declaration::UiSelectionPolicy::single())
}

pub(super) fn launch_query_world() -> (
    super::session::World,
    worth_ui_query_binding::UiProjectionOptionReference,
) {
    use worth_ui_query_binding::*;

    let (mut query, _) = certification::seeded_collection_projection_workspace_with_item_keys(
        vec![("selected".into(), "Selected".into(), 7)],
        certification::WorthUiCollectionProjectionSeedPosture::Complete,
    );
    let domain = query.worth_ui().unwrap();
    let registration =
        UiCollectionProjectionRegistration::text(
            domain.projection_view(SELECTION_PROJECTION).unwrap(),
            UiProjectionFieldRequirement::identity_id(),
            [UiProjectionFieldRequirement::query_text_status()],
            false,
            false,
        )
        .unwrap()
        .with_unsigned64_application_item_key_field(
            UiProjectionFieldRequirement::collection_item_key(),
        );
    let UiCollectionProjectionBindingAdmission::Ready(binding) = registration.clone().admit(&query)
    else {
        panic!("AP-07 Query binding admits")
    };
    let UiCollectionProjectionOpenOutcome::Opened(opened) = binding.open(
        UiCollectionProjectionBudget::new(1, 1, 0, 512).unwrap(),
        &mut query,
    ) else {
        panic!("AP-07 collection opens")
    };
    let (_, fact) = opened.into_parts();
    let UiProjectionAvailability::Present(UiPresentProjection::Current(value)) =
        fact.availability()
    else {
        panic!("AP-07 collection is current")
    };
    let row = value.rows()[0].row().clone();
    let projection = registration.view().identity().clone();
    let mut world =
        super::session::World::launch_ap07(registration, super::authored::ap07_bootstrap_source());
    let frame = world.prepare();
    world.publish(frame, 1, true);
    world.session.unmount_instance(world.instances[1]).unwrap();
    let initial_regions = regions(&world);
    super::geometry::install_without_target(
        &mut world.session,
        world.surfaces,
        world.instances,
        initial_regions,
    );
    world.session.advance_mounted_identity_frame().unwrap();
    cutover_six_axis(
        &mut world,
        UiProjectionObservation::Collection(fact.into_observation()),
    );
    let option = world
        .session
        .current_projection_option(&projection, &row)
        .unwrap();
    (world, option)
}

pub(super) fn establish_six_axis_target(
    world: &mut super::session::World,
    option: worth_ui_query_binding::UiProjectionOptionReference,
) {
    let graph = world
        .session
        .graph()
        .node_identities()
        .find(|identity| {
            world
                .session
                .graph()
                .lookup()
                .graph_node(*identity)
                .is_some_and(|node| {
                    node.value().declaration_identity().authored_semantic_name()
                        == format!("component:{}", super::authored::COMPONENTS[1])
                })
        })
        .unwrap();
    world.graphs[1] = graph;
    let handle = world.session.mounted_graph_node(graph).unwrap();
    world.instances[1] = world
        .session
        .mount_instance(handle, world.surfaces[0])
        .unwrap();
    let regions = regions(world);
    super::geometry::install_successor(
        &mut world.session,
        world.surfaces,
        world.instances,
        regions,
    );
    world.session.advance_mounted_identity_frame().unwrap();
    let owner = receipt(world, world.instances[0]);
    let target = receipt(world, world.instances[1]);
    world
        .session
        .bind_selection_item(owner, target, option)
        .unwrap();
    let validation = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        &world.session,
        graph,
        world.instances[1],
        target,
    )
    .unwrap();
    world
        .session
        .intent_application_facts
        .publish_validation_appearance_fact(
            validation,
            None,
            crate::runtime::intent::UiValidationAppearanceClass::Valid,
        )
        .unwrap();
    let mapping = world
        .session
        .mounted
        .selection_mapping_for_item(world.instances[1])
        .unwrap();
    world
        .session
        .selection
        .as_mut()
        .unwrap()
        .apply(
            mapping.owner,
            mapping.incarnation,
            crate::runtime::selection::UiSelectionRequest::SelectSingle(mapping.key),
        )
        .unwrap();
    close_owner_snapshot(world);
    let frame = world.prepare_surface(world.surfaces[0]);
    world.publish(frame, 6, true);
    super::captured_geometry::evaluate_target_operability(world, 1);
    let focus = world.session.focus.as_mut().unwrap();
    let scope = focus.default_scope_for_surface(world.surfaces[0]).unwrap();
    let mut current = None;
    for _ in 0..world.instances.len() {
        current = focus
            .commit_host_traversal(
                scope,
                crate::runtime::focus::UiHostFocusTraversalDirection::Forward,
                false,
            )
            .unwrap()
            .current()
            .map(|target| target.mounted_instance());
        if current == Some(world.instances[1]) {
            break;
        }
    }
    assert_eq!(current, Some(world.instances[1]));
    close_owner_snapshot(world);
}

pub(super) fn close_owner_snapshot(world: &mut super::session::World) {
    let turn = world.session.begin_observation_turn().unwrap();
    let admitted = turn.seal().unwrap();
    world.session.classify_observations(admitted).unwrap();
}

fn cutover_six_axis(
    world: &mut super::session::World,
    query: worth_ui_query_binding::UiProjectionObservation,
) {
    let source = super::authored::ap07_source();
    let candidate = crate::runtime::WorthUiReloadDebounce::default()
        .debounce(
            crate::runtime::WorthUiSourceProvider::in_memory("overlay-appearance-runtime")
                .with_file("app/main.wui", source),
            &[crate::runtime::WorthUiWatcherEvent::provider_revision(
                "overlay-appearance-runtime",
            )],
            2,
        )
        .unwrap()
        .attempt_candidate_for_certification(world.session.capabilities())
        .unwrap();
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_projection_query(query).unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let crate::facade::observation::UiChangeClassificationOutcome::Changed(changed) =
        world.session.classify_observations(admitted).unwrap()
    else {
        panic!("six-axis cutover changes meaning")
    };
    let lifecycle = world
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = world
        .session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let prepared = world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(5),
        )
        .unwrap();
    for _ in prepared.prepared_frame().unwrap().surfaces() {
        world.host.push_native_display_settled_without_effects();
    }
    match prepared.execute(5) {
        crate::runtime::rebind::UiRebindOutcome::Published(_) => {}
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "six-axis cutover denied: {:?}, {:?}",
            denial.cause(),
            denial.host_rejections()
        ),
        crate::runtime::rebind::UiRebindOutcome::InFlight(_) => {
            panic!("six-axis cutover remained in flight")
        }
        crate::runtime::rebind::UiRebindOutcome::Indeterminate(_) => {
            panic!("six-axis cutover became indeterminate")
        }
        crate::runtime::rebind::UiRebindOutcome::InternalDefect(defect) => {
            panic!("six-axis cutover internal defect: {:?}", defect.kind())
        }
        crate::runtime::rebind::UiRebindOutcome::Duplicate(_)
        | crate::runtime::rebind::UiRebindOutcome::ObservedNoChange(_)
        | crate::runtime::rebind::UiRebindOutcome::CancelledBeforeEffects(_)
        | crate::runtime::rebind::UiRebindOutcome::TimedOutBeforeEffects(_)
        | crate::runtime::rebind::UiRebindOutcome::SupersededBeforeEffects(_) => {
            panic!("six-axis cutover stopped before publication")
        }
    }
}

pub(super) fn regions(
    world: &super::session::World,
) -> [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2] {
    let bindings = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    ["workspace.surface.overlay", "workspace.surface.secondary"].map(|surface| {
        bindings
            .region_named(surface, "workspace.region.primary")
            .unwrap()
    })
}

fn receipt(
    world: &super::session::World,
    instance: UiMountedInstanceIdentity,
) -> UiMountedNodeReceiptIdentity {
    world
        .session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .find(|row| row.mounted_instance_identity() == instance)
        .unwrap()
        .node_receipt_identity()
}

pub(super) fn reject_stale_duplicate_and_foreign_bases(
    world: &mut super::session::World,
    sequence: u64,
) {
    let surface = world.surfaces[0];
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let wrong_epoch = UiHostObservationPresentationBasis::new(
        current.host_surface(),
        current.frame(),
        current.binding(),
        UiHostPresentationEpoch::issued_by_host(current.epoch().diagnostic_value().wrapping_add(1)),
    );
    let frame = world.session.current_mounted_publication().unwrap().frame();
    let appearance = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let session = world.session.host_session.identity().as_u64();
    let batch = |host_session, presentation| {
        super::pointer_geometry::pointer_batch(
            host_session,
            presentation,
            sequence,
            [150_000, 55_000],
        )
    };

    assert_eq!(
        world
            .session
            .validate_host_observation_batch(batch(session + 1, current)),
        UiHostObservationReportOutcome::Denied(UiHostObservationReportDenial::ForeignHostSession)
    );
    assert_eq!(
        world
            .session
            .validate_host_observation_batch(batch(session, wrong_epoch)),
        UiHostObservationReportOutcome::Denied(
            UiHostObservationReportDenial::PresentationEpochMismatch
        )
    );
    let current_batch = batch(session, current);
    assert!(matches!(
        world
            .session
            .validate_host_observation_batch(current_batch.clone()),
        UiHostObservationReportOutcome::Validated(_)
    ));
    assert!(matches!(
        world.session.validate_host_observation_batch(current_batch),
        UiHostObservationReportOutcome::Duplicate(_)
    ));
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        frame
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap(),
        appearance.as_ref(),
        "hostile bases cannot replace accepted appearance"
    );
}
