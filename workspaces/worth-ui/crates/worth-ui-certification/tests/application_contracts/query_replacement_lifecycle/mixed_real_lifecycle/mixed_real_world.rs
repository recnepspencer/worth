//! The mounted world a real file-authored cross-lane application runs in:
//! its component mounts, first allocation, Query projection and frames.

use worth_ui::facade::app::WorthUiVisibleRange;
use worth_ui::facade::declaration::UiDeclarationStructuralRole;
use worth_ui::facade::measurement_exchange::{
    UiMeasurementEvidenceFamily, UiViewportExtentRequest,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext, WorthUiHostCapability,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedAllocationCertificationExt, WorthUiMountedIdentityCertificationExt,
};

use crate::mounted_application_lifecycle::adapter_projection_world::preview_target;
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;

pub(super) fn mount_all_nodes(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> (
    worth_ui::facade::graph::UiGraphNodeIdentity,
    worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity,
) {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let target = preview_target(session);
    // Production never mounts the bootstrap page root. The authored surface
    // is mounted because its splitter region is the preview target.
    let graph = session.graph();
    let nodes = graph
        .node_identities()
        .filter(|node| {
            graph.lookup().graph_node(*node).is_some_and(|record| {
                record.value().structural_role() != UiDeclarationStructuralRole::Page
            })
        })
        .collect::<Vec<_>>();
    let mut preview_instance = None;
    for node in nodes {
        let handle = session.mounted_graph_node(node).unwrap();
        let instance = session.mount_instance(handle, surface).unwrap();
        if node == target {
            preview_instance = Some(instance);
        }
    }
    (
        target,
        preview_instance.expect("file-authored splitter node is mounted"),
    )
}

pub(super) fn establish_first_allocation_catalog(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let capability = session.host_measurement_capability();
    assert!(capability
        .capability_report()
        .supports(WorthUiHostCapability::ViewportObservation));
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let input = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::ViewportExtent,
        UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    let receipt = session
        .establish_mounted_allocation_catalog(1, [input])
        .expect("mounted graph and real host measurement establish the first catalog");
    let committed = receipt.committed();
    assert!(!committed.receipts().is_empty());
    assert_eq!(
        usize::from(committed.counters().committed_receipts()),
        committed.receipts().len()
    );
}

pub(super) fn admit_query_projection(
    scenario: &mut FilesystemApplicationLifecycleScenario,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let projection = scenario.settled_query_projection();
    let link = session
        .query_fact_link("inspector.measurements")
        .expect("file-authored binding resolves");
    drop(
        session
            .execute_framework_turn(|turn| {
                turn.query_projection(|source| {
                    source.admit_settled(projection).unwrap();
                    source.submit_settled(&link).unwrap();
                });
            })
            .expect("query projection enters outside mounted presentation")
            .into_completion(),
    );
}

pub(super) fn publish_all_lane_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let outcome = session
        .execute_mounted_frame(
            all_lane_request(),
            UiPresentationDeadline::at_tick(20),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("public mounted facade executes the all-lane frame"));
    match outcome {
        UiMountedFrameOutcome::Published(publication) => {
            assert_every_instance_published(session, &publication);
            publication
        }
        UiMountedFrameOutcome::Unchanged(_) => panic!("all-lane frame was unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("all-lane frame only reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => panic!(
            "all-lane frame was rejected before effects: {:?}",
            rejected.rejections()
        ),
        UiMountedFrameOutcome::InFlight(_) => panic!("all-lane frame remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("all-lane frame became indeterminate")
        }
        UiMountedFrameOutcome::Superseded(_) => panic!("all-lane frame was superseded"),
        UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("all-lane frame retention denied: {:?}", rejection.denial())
        }
        UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("all-lane frame admission denied: {:?}", rejection.denial())
        }
        UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("all-lane frame completion denied: {denial:?}")
        }
    }
}

pub(super) fn all_lane_request() -> UiMountedFrameRequest {
    UiMountedFrameRequest::all_bound_surfaces()
        .with_virtualized_range(WorthUiVisibleRange::rows(0, 1).unwrap())
}

/// Every mounted instance, whatever its lane, has a receipt in the frame the
/// host now shows.
fn assert_every_instance_published(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    publication: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
) {
    let inspection = session.inspect_mounted_identity();
    assert_eq!(inspection.current_frame(), Some(publication.frame()));
    for instance in inspection.mounted_instances() {
        assert!(
            inspection.frame_receipts().iter().any(|receipt| {
                receipt.frame_identity() == publication.frame()
                    && receipt.mounted_instance_identity() == instance.identity()
            }),
            "mounted instance {:?} has no receipt in the published frame",
            instance.identity()
        );
    }
}
