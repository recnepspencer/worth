use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiMountedIdentityDenial, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;
use worth_ui_test_support::WorthUiServiceStateCertificationExt;

use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::in_flight_presentation_world::{
    mounted_session, prepared as prepared_frame,
};
use super::mounted_application_lifecycle::known_empty_surface_world::{
    active_session, first_node, profile, registered_surface,
};
use super::mounted_host_protocol::scripted_host::ScriptedPresentationHost;
use super::mounted_publication::{replacement_workspace, stage_replacement};

#[test]
fn zero_many_reorder_and_remount_preserve_only_lawful_identity() {
    let mut session = active_session();
    let surface = registered_surface(&mut session);
    let node = first_node(&session);
    assert!(session.mounted_instances_for(node).unwrap().is_empty());

    let first = session.mount_instance(node, surface).unwrap();
    let second = session.mount_instance(node, surface).unwrap();
    let first_incarnation = incarnation(&session, first);
    let second_incarnation = incarnation(&session, second);
    assert_ne!(first_incarnation, second_incarnation);
    assert_eq!(session.mounted_instances_for(node).unwrap().len(), 2);

    let first_frame = session.advance_mounted_identity_frame().unwrap();
    let first_receipts = receipt_ids(&session);
    session.reorder_mounted_instances(&[second, first]).unwrap();
    let reordered = session.inspect_mounted_identity();
    assert_eq!(
        reordered
            .mounted_instances()
            .iter()
            .map(|entry| entry.identity())
            .collect::<Vec<_>>(),
        vec![second, first]
    );
    assert_eq!(incarnation(&session, first), first_incarnation);
    let second_frame = session.advance_mounted_identity_frame().unwrap();
    assert_ne!(first_frame, second_frame);
    assert_ne!(first_receipts, receipt_ids(&session));

    session.unmount_instance(first).unwrap();
    let remounted = session.mount_instance(node, surface).unwrap();
    assert_ne!(first, remounted);
    assert_ne!(first_incarnation, incarnation(&session, remounted));
    assert_eq!(
        session.unmount_instance(first),
        Err(UiMountedIdentityDenial::RetiredMountedInstance)
    );
}

#[test]
fn foreign_world_surface_and_binding_values_open_no_mounted_doors() {
    let mut left = active_session();
    let mut right = active_session();
    let left_surface = registered_surface(&mut left);
    let right_surface = registered_surface(&mut right);
    let right_binding = right.inspect_mounted_identity().surface_bindings()[0];
    let left_node = first_node(&left);
    let right_node = first_node(&right);

    assert_eq!(
        left.mount_instance(right_node, left_surface),
        Err(UiMountedIdentityDenial::ForeignGraphWorld)
    );
    assert_eq!(
        left.mount_instance(left_node, right_surface),
        Err(UiMountedIdentityDenial::UnknownSemanticSurface)
    );
    let right_instance = right.mount_instance(right_node, right_surface).unwrap();
    assert_eq!(
        left.unmount_instance(right_instance),
        Err(UiMountedIdentityDenial::UnknownMountedInstance)
    );
    assert_eq!(
        left.validate_current_surface_binding(right_binding.binding_generation()),
        Err(UiMountedIdentityDenial::UnknownSurfaceBinding)
    );

    let left_instance = left.mount_instance(left_node, left_surface).unwrap();
    let left_frame = left.advance_mounted_identity_frame().unwrap();
    let left_receipt = receipt_ids(&left)[0];
    let right_frame = right.advance_mounted_identity_frame().unwrap();
    let right_receipt = receipt_ids(&right)[0];
    assert!(left.validate_current_mounted_frame(left_frame).is_ok());
    assert!(left
        .validate_current_mounted_node_receipt(left_instance, left_receipt)
        .is_ok());
    assert_eq!(
        left.validate_current_mounted_frame(right_frame),
        Err(UiMountedIdentityDenial::FrameNotCurrent)
    );
    assert_eq!(
        left.validate_current_mounted_node_receipt(left_instance, right_receipt),
        Err(UiMountedIdentityDenial::NodeReceiptNotCurrent)
    );
    let unadmitted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    assert_eq!(
        left.unmount_instance(unadmitted_instance),
        Err(UiMountedIdentityDenial::UnknownMountedInstance)
    );
    let unadmitted_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    assert_eq!(
        left.mount_instance(left_node, unadmitted_surface),
        Err(UiMountedIdentityDenial::UnknownSemanticSurface)
    );
}

#[test]
fn application_replacement_advances_the_world_and_preserves_uninterrupted_mounts() {
    let host = ScriptedPresentationHost::default();
    let (mut session, _) = mounted_session(host.clone(), "mounted-replacement", 1);
    let surface =
        session.inspect_mounted_identity().surface_bindings()[0].semantic_surface_identity();
    let predecessor_node = first_node(&session);
    let predecessor_instance = session.inspect_mounted_identity().mounted_instances()[0].identity();
    host.push_presented();
    let initial_frame = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(
            initial_frame,
            UiPresentationDeadline::at_tick(10),
            0,
        ),
        UiMountedFrameOutcome::Published(_)
    ));

    let workspace = replacement_workspace("mounted-replacement");
    let (pending, catalog, boundary) = stage_replacement(&workspace, &mut session);
    let prepared = match session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap()
    {
        worth_ui::facade::app::WorthUiMountedReplacementPreparationOutcome::Prepared(value) => {
            value
        }
        _ => panic!("changed application meaning requires activation"),
    };
    host.push_presented();
    let (cutover, mounted) = match prepared.present(UiPresentationDeadline::at_tick(20), 1) {
        worth_ui::facade::app::WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted,
        } => (application, mounted),
        _ => panic!("complete presentation publishes the changed application"),
    };
    assert_eq!(cutover.active_generation(), session.generation_identity());

    assert_eq!(
        session.mounted_instances_for(predecessor_node),
        Err(UiMountedIdentityDenial::ForeignGraphWorld)
    );
    assert!(
        session
            .graph()
            .node_identities()
            .any(|identity| identity == predecessor_node.graph_node_identity()),
        "the denial must come from the stale mounted world, not a missing graph node"
    );
    let successor_node = session
        .mounted_graph_node(predecessor_node.graph_node_identity())
        .expect("preserved semantic node receives a successor-world handle");
    assert_eq!(
        session
            .mounted_instances_for(successor_node)
            .unwrap()
            .as_ref(),
        &[predecessor_instance],
        "uninterrupted semantic mounts survive graph-world replacement"
    );
    let current = session.inspect_mounted_identity();
    assert_eq!(current.mounted_instances().len(), 1);
    assert_eq!(current.current_frame(), Some(mounted.frame()));
    assert_eq!(current.surface_bindings().len(), 1);
    assert_eq!(
        current.surface_bindings()[0].semantic_surface_identity(),
        surface
    );
    drop(cutover);
    workspace.close();
}

#[test]
fn application_replacement_retires_one_removed_scroll_neighborhood_and_retains_its_sibling() {
    let host = ScriptedPresentationHost::default();
    let initial_workspace = replacement_workspace("mounted-replacement-retirement-initial");
    let initial_snapshot = WorthUiFilesystemSourceProvider::new(initial_workspace.root())
        .read()
        .expect("two-component predecessor source settles");
    let scenario = FilesystemApplicationLifecycleScenario::new("mounted replacement retirement");
    let capabilities = scenario.capability_application();
    let initial_submission = initial_snapshot
        .attempt_candidate_for_certification(capabilities.capabilities())
        .expect("two-component predecessor lowers");
    let mut session = scenario
        .prepare_application_with_host(initial_submission, host.clone())
        .launch()
        .expect("two-component predecessor launches");
    let expected_components = [
        "component:workspace.component.authority_candidate",
        "component:workspace.component.authority_current",
    ];
    let graph = session.graph();
    let mut declared_components = graph
        .node_identities()
        .filter_map(|identity| {
            let record = graph.lookup().graph_node(identity)?;
            let value = record.value();
            let declaration = value.declaration_identity().authored_semantic_name();
            expected_components
                .contains(&declaration)
                .then_some((declaration.to_owned(), identity))
        })
        .collect::<Vec<_>>();
    declared_components.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        declared_components
            .iter()
            .map(|(declaration, _)| declaration.as_str())
            .collect::<Vec<_>>(),
        expected_components,
        "the predecessor exposes exactly the retained and removed authored components"
    );
    let nodes = declared_components
        .into_iter()
        .map(|(_, identity)| {
            session
                .mounted_graph_node(identity)
                .expect("declared component has a mounted handle")
        })
        .collect::<Vec<_>>();
    let predecessor_neighborhoods = nodes.len();
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    for node in nodes {
        session.mount_instance(node, surface).unwrap();
    }
    host.push_presented();
    let initial_frame = prepared_frame(&mut session);
    assert!(matches!(
        session.present_prepared_mounted_frame(
            initial_frame,
            UiPresentationDeadline::at_tick(10),
            0,
        ),
        UiMountedFrameOutcome::Published(_)
    ));
    let scroll_before = session.inspect_scroll_runtime_for_certification();
    assert_eq!(
        scroll_before.ownership_instances(),
        predecessor_neighborhoods,
        "mount-time ownership discovery installs every predecessor catalog record"
    );

    let workspace = FilesystemContractWorkspace::new("mounted-replacement-retirement");
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
    );
    let (pending, catalog, boundary) = stage_replacement(&workspace, &mut session);
    let prepared = match session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap()
    {
        worth_ui::facade::app::WorthUiMountedReplacementPreparationOutcome::Prepared(value) => {
            value
        }
        _ => panic!("removing one predecessor component requires activation"),
    };
    host.push_presented();
    assert!(matches!(
        prepared.present(UiPresentationDeadline::at_tick(20), 1),
        worth_ui::facade::app::WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));

    let successor_neighborhoods = session.inspect_mounted_identity().mounted_instances().len();
    assert!(
        successor_neighborhoods > 0 && successor_neighborhoods < predecessor_neighborhoods,
        "the successor retains a sibling while removing at least one neighborhood"
    );
    let scroll = session.inspect_scroll_runtime_for_certification();
    assert_eq!(
        (scroll.ownership_instances(), scroll.owners()),
        (successor_neighborhoods, 0),
        "the prepared successor retires exactly the removed mounted ownership records"
    );
    assert_eq!(
        scroll.ownership_resolutions(),
        scroll_before.ownership_resolutions()
            + u64::try_from(successor_neighborhoods).expect("bounded mounted count"),
        "replacement discovery revisits only retained successor neighborhoods"
    );
    assert_eq!(
        scroll.ownership_incarnations().len(),
        successor_neighborhoods
    );
    workspace.close();
    initial_workspace.close();
}

pub(super) fn incarnation(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    identity: worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) -> worth_ui_runtime::facade::mounted::UiMountIncarnation {
    session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .find(|entry| entry.identity() == identity)
        .unwrap()
        .mount_incarnation()
}

pub(super) fn receipt_ids(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> Vec<worth_ui_runtime::facade::mounted::UiMountedNodeReceiptIdentity> {
    session
        .inspect_mounted_identity()
        .frame_receipts()
        .iter()
        .map(|entry| entry.node_receipt_identity())
        .collect()
}
