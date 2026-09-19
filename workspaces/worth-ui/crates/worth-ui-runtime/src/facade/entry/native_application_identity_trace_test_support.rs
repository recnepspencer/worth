use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedFrameRequest,
    UiMountedIdentityView,
};

use super::native_identity_trace_audit::{
    audit_retained_identity_trace, prepared_identity_trace_oracles, PreparedIdentityTraceOracle,
    RetainedIdentityTraceAudit, RetainedIdentityTraceAuditCost,
};
use super::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
    WorthUiNativeApplicationShell,
};
use crate::facade::WorthUiPreparedMountedApplicationReplacement;

pub(super) type CandidateProvenanceOracle = Vec<PreparedIdentityTraceOracle>;

pub(super) fn install_bound_surface_geometry(shell: &mut WorthUiNativeApplicationShell) {
    let surfaces = shell
        .session
        .mounted
        .view()
        .surface_bindings()
        .iter()
        .map(|binding| binding.semantic_surface_identity())
        .collect::<Vec<_>>();
    for surface in surfaces {
        crate::facade::entry::mounted_occurrence_geometry_test_support::
            refresh_nonoverlapping_surface_geometry(&mut shell.session, surface);
    }
}

pub(super) fn frame_receipt(outcome: UiMountedFrameOutcome) -> UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!(
                "mounted frame was rejected before effects: {:?}",
                rejected.rejections()
            )
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("mounted frame remained in flight"),
        UiMountedFrameOutcome::Superseded(_) => panic!("mounted frame was superseded"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("mounted frame presentation became indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => {
            panic!("mounted frame retention was denied")
        }
        UiMountedFrameOutcome::AdmissionDenied(_) => {
            panic!("mounted frame admission was denied")
        }
        UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("mounted frame completion was denied")
        }
    }
}

pub(super) fn completed(
    outcome: Result<
        UiMountedFrameOutcome,
        crate::facade::entry::WorthUiMountedFrameExecutionStop<'_>,
    >,
) -> UiMountedFrameOutcome {
    match outcome {
        Ok(frame) => frame,
        Err(_) => panic!("mounted frame should complete"),
    }
}

pub(super) fn authored_presented_identity(
    shell: &WorthUiNativeApplicationShell,
    receipt: &UiMountedFramePublicationReceipt,
    authored_provenance: u64,
    relation: &str,
) -> crate::mounting::UiMountedFrameIdentityView {
    let mounted = shell.session.mounted.view();
    let candidates = mounted
        .frame_receipts()
        .iter()
        .copied()
        .filter(|identity| identity.frame_identity() == receipt.frame())
        .collect::<Vec<_>>();
    let examined = candidates
        .iter()
        .map(|identity| {
            let instance = mounted_instance(&mounted, identity.mounted_instance_identity());
            let record = shell
                .session
                .graph()
                .lookup()
                .graph_node(instance.graph_node_identity())
                .expect("mounted graph node should remain graph-addressable");
            (
                *identity,
                instance.graph_node_identity(),
                record.value().authored_provenance_digest(),
            )
        })
        .collect::<Vec<_>>();
    examined
        .iter()
        .find(|(_, _, provenance)| *provenance == authored_provenance)
        .map(|(identity, _, _)| *identity)
        .unwrap_or_else(|| {
            panic!(
                "{relation} frame did not carry target provenance {authored_provenance:#x}; examined {examined:?}"
            )
        })
}

pub(super) fn mounted_source_oracle(
    artifacts: &[crate::declaration::UiDeclarationArtifact],
    graph: crate::graph::UiGraphAuthority<'_>,
) -> PreparedIdentityTraceOracle {
    prepared_identity_trace_oracles(artifacts, graph)
        .into_iter()
        .find(|oracle| oracle.authored_provenance().source_artifact().path() == "app/main.wui")
        .expect("app/main.wui should contribute a graph-backed declaration")
}

pub(super) fn replace_application_with_provenance(
    shell: &mut WorthUiNativeApplicationShell,
    submission: crate::runtime::WorthUiWatchedCandidateSubmission,
) -> (UiMountedFramePublicationReceipt, CandidateProvenanceOracle) {
    let mut prepared = shell
        .session
        .prepare_replacement(submission)
        .expect("candidate source should prepare");
    let catalog = shell
        .session
        .admit_native_replacement_allocation_catalog(&mut prepared)
        .expect("candidate allocation should establish through the native host");
    let provenance = prepared_identity_trace_oracles(
        prepared.candidate_declaration_artifacts(),
        prepared.candidate_graph(),
    );
    let lowered = shell
        .session
        .lower_prepared_replacement(*prepared)
        .expect("prepared candidate should lower");
    let pending = shell
        .session
        .stage_prepared_replacement(lowered)
        .expect("lowered candidate should stage");
    let boundary = shell
        .session
        .execute_framework_turn(|_| {})
        .unwrap_or_else(|_| panic!("replacement boundary should be available"))
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement boundary should execute"))
        .into_activation_boundary();
    let prepared = shell
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("mounted replacement should prepare");
    let replacement = match prepared {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("changed source must prepare a mounted replacement")
        }
    };
    (publish_prepared_replacement(replacement), provenance)
}

fn publish_prepared_replacement(
    replacement: Box<WorthUiPreparedMountedApplicationReplacement<'_>>,
) -> UiMountedFramePublicationReceipt {
    match replacement.present(
        worth_ui_host_contract::UiPresentationDeadline::at_tick(200),
        2,
    ) {
        WorthUiMountedApplicationReplacementOutcome::Published { mounted, .. } => mounted,
        WorthUiMountedApplicationReplacementOutcome::RejectedBeforeEffects(_) => {
            panic!("replacement was rejected before effects")
        }
        WorthUiMountedApplicationReplacementOutcome::InFlight(_) => {
            panic!("replacement remained in flight")
        }
        WorthUiMountedApplicationReplacementOutcome::PresentationIndeterminate(_) => {
            panic!("replacement presentation became indeterminate")
        }
        WorthUiMountedApplicationReplacementOutcome::RetentionDenied(_) => {
            panic!("replacement retention was denied")
        }
        WorthUiMountedApplicationReplacementOutcome::AdmissionDenied(_) => {
            panic!("replacement admission was denied")
        }
        WorthUiMountedApplicationReplacementOutcome::CompletionDenied(_) => {
            panic!("replacement completion was denied")
        }
    }
}

pub(super) fn only_presented_identity(
    shell: &WorthUiNativeApplicationShell,
    receipt: &UiMountedFramePublicationReceipt,
) -> crate::mounting::UiMountedFrameIdentityView {
    let identities = shell
        .session
        .mounted
        .view()
        .frame_receipts()
        .iter()
        .copied()
        .filter(|identity| identity.frame_identity() == receipt.frame())
        .collect::<Vec<_>>();
    assert_eq!(
        identities.len(),
        1,
        "replacement fixture should publish one surviving mounted declaration"
    );
    identities[0]
}

pub(super) fn presented_graph_node(
    shell: &WorthUiNativeApplicationShell,
    identity: crate::mounting::UiMountedFrameIdentityView,
) -> crate::graph::UiGraphNodeIdentity {
    let mounted = shell.session.mounted.view();
    mounted_instance(&mounted, identity.mounted_instance_identity()).graph_node_identity()
}

pub(super) fn mounted_instance(
    view: &UiMountedIdentityView,
    identity: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> crate::mounting::UiMountedInstanceIdentityView {
    view.mounted_instances()
        .iter()
        .find(|candidate| candidate.identity() == identity)
        .cloned()
        .expect("published mounted instance should remain in the selected view")
}

pub(super) fn retained_trace(
    shell: &WorthUiNativeApplicationShell,
    receipt: &UiMountedFramePublicationReceipt,
    identity: crate::mounting::UiMountedFrameIdentityView,
) -> RetainedIdentityTraceAudit {
    let capture = shell
        .session
        .mounted
        .acquire_visual_snapshot(receipt.frame(), receipt.bindings()[0])
        .expect("current or retained predecessor should expose its exact visual basis");
    let (_lease, _regions, trace_basis) = capture.into_parts();
    audit_retained_identity_trace(
        &trace_basis,
        identity.mounted_instance_identity(),
        identity.node_receipt_identity(),
    )
}

pub(super) fn remount_presented_instance(
    shell: &mut WorthUiNativeApplicationShell,
    predecessor_identity: crate::mounting::UiMountedFrameIdentityView,
) -> (
    UiMountedFramePublicationReceipt,
    worth_ui_host_contract::UiMountedInstanceIdentity,
    worth_ui_host_contract::UiMountIncarnation,
) {
    let predecessor_view = mounted_instance(
        &shell.session.mounted.view(),
        predecessor_identity.mounted_instance_identity(),
    );
    let graph_node = predecessor_view.graph_node_identity();
    let predecessor_incarnation = predecessor_view.mount_incarnation();
    let surface = shell.session.mounted.view().surface_bindings()[0].semantic_surface_identity();
    shell
        .session
        .unmount_instance(predecessor_identity.mounted_instance_identity())
        .expect("the presented instance should unmount");
    let handle = shell
        .session
        .mounted_graph_node(graph_node)
        .expect("the same graph node should remain mountable");
    let successor_instance = shell
        .session
        .mount_instance(handle, surface)
        .expect("the same graph node should remount");
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut shell.session, surface);
    let successor = frame_receipt(completed(shell.present_frame(200, 2)));
    (successor, successor_instance, predecessor_incarnation)
}

pub(super) fn assert_trace_runtime_identity(
    trace: &RetainedIdentityTraceAudit,
    identity: crate::mounting::UiMountedFrameIdentityView,
) {
    assert_eq!(
        trace.mounted_instance(),
        identity.mounted_instance_identity()
    );
    assert_eq!(trace.node_receipt(), identity.node_receipt_identity());
}

pub(super) fn assert_trace_authored_affinity(
    trace: &RetainedIdentityTraceAudit,
    oracle: &PreparedIdentityTraceOracle,
) {
    assert_eq!(trace.declaration(), oracle.declaration());
    assert_eq!(trace.authored_provenance(), oracle.authored_provenance());
    assert_eq!(trace.evidence(), oracle.evidence());
}

pub(super) fn assert_same_authored_affinity(
    predecessor: &RetainedIdentityTraceAudit,
    successor: &RetainedIdentityTraceAudit,
) {
    assert_eq!(predecessor.graph_node(), successor.graph_node());
    assert_eq!(predecessor.declaration(), successor.declaration());
    assert_eq!(
        predecessor.authored_provenance(),
        successor.authored_provenance()
    );
    assert_eq!(predecessor.evidence(), successor.evidence());
    assert_eq!(predecessor.generation(), successor.generation());
}

pub(super) fn assert_indexed_trace_cost(cost: RetainedIdentityTraceAuditCost) {
    assert!(cost.mounted_receipt_index_probes() > 0);
    assert!(cost.mounted_node_index_probes() > 0);
    assert_eq!(cost.graph_identity_index_lookups(), 1);
    assert_eq!(cost.declaration_artifact_index_lookups(), 1);
    assert_eq!(cost.declaration_identity_index_lookups(), 1);
    assert_eq!(cost.authored_provenance_index_lookups(), 1);
    assert!(cost.trace_index_probes() <= 16);
}
