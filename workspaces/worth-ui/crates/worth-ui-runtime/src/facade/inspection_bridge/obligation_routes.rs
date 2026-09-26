use crate::admission::{UiAdmissionTarget, UiAdmissionWorld};
use crate::facade::inspection_bridge::boundary_access::declaration_artifact_for_graph_node;
use crate::facade::inspection_bridge::UiInspectionReceipt;
use crate::facade::WorthUiApp;
use crate::graph::UiGraphNodeIdentity;
use crate::obligations::touch::{
    UiGraphTouchAspectPosture, UiGraphTouchAspects, UiGraphTouchDenial, UiGraphTouchDescriptor,
    UiGraphTouchTiming,
};
use worth_ui_inspection::{UiInspectionQuery, UiInspectionTarget};

pub(crate) fn inspect_retained_obligation_query(
    app: &WorthUiApp,
    query: UiInspectionQuery,
) -> Option<UiInspectionReceipt> {
    match query.target() {
        UiInspectionTarget::ObligationEvidenceHandle { handle_digest } => {
            let selected = app
                .retained_obligation_registry()
                .retained_selection(*handle_digest)?;
            let receipt = selected.inspect(query);
            app.retained_obligation_registry()
                .register(&selected, &receipt);
            Some(receipt)
        }
        UiInspectionTarget::ObligationTouch {
            graph_node_digest,
            touch_identity_digest,
        } => canonical_touch_for_node(app, *graph_node_digest, Some(*touch_identity_digest))
            .map(|touch| inspect_selected_obligations(app, touch, query)),
        UiInspectionTarget::ObligationGraphNode { graph_node_digest } => {
            canonical_touch_for_node(app, *graph_node_digest, None)
                .map(|touch| inspect_selected_obligations(app, touch, query))
        }
        UiInspectionTarget::ProductRoot
        | UiInspectionTarget::DeclaredSurface { .. }
        | UiInspectionTarget::GraphNodeIdentity { .. }
        | UiInspectionTarget::PublishedAspect { .. }
        | UiInspectionTarget::ConsumedAspect { .. }
        | UiInspectionTarget::DeclarationIdentity { .. }
        | UiInspectionTarget::AuthoredSourceProvenance { .. }
        | _ => None,
    }
}

fn inspect_selected_obligations(
    app: &WorthUiApp,
    touch: UiGraphTouchDescriptor,
    query: UiInspectionQuery,
) -> UiInspectionReceipt {
    let target = UiAdmissionTarget::graph_node(
        touch.target().graph_node_identity(),
        UiAdmissionWorld::from_graph_world_profile(touch.world().world_profile().clone()),
    );
    let selected = app
        .admission()
        .select_obligations_for_target(&touch, target);
    let receipt = selected.inspect(query);
    app.retained_obligation_registry()
        .register(&selected, &receipt);
    receipt
}

fn canonical_touch_for_node(
    app: &WorthUiApp,
    graph_node_digest: u64,
    expected_touch_identity_digest: Option<u64>,
) -> Option<UiGraphTouchDescriptor> {
    let graph_node_identity = UiGraphNodeIdentity::new(graph_node_digest);
    let query_touch = query_touch_for_node(app, graph_node_identity);
    if let Some(expected_digest) = expected_touch_identity_digest {
        if query_touch
            .as_ref()
            .is_some_and(|touch| touch.identity_digest() == expected_digest)
        {
            return query_touch;
        }
    } else if query_touch.is_some() {
        return query_touch;
    }

    structural_touch_for_node(app, graph_node_identity).filter(|touch| {
        expected_touch_identity_digest
            .is_none_or(|expected_digest| touch.identity_digest() == expected_digest)
    })
}

fn query_touch_for_node(
    app: &WorthUiApp,
    graph_node_identity: UiGraphNodeIdentity,
) -> Option<UiGraphTouchDescriptor> {
    try_query_touch_for_node(app, graph_node_identity).ok()
}

pub(crate) fn try_query_touch_for_node(
    app: &WorthUiApp,
    graph_node_identity: UiGraphNodeIdentity,
) -> Result<UiGraphTouchDescriptor, UiGraphTouchDenial> {
    let graph = app.graph();
    let control_node = graph
        .lookup()
        .graph_node(graph_node_identity)
        .ok_or(UiGraphTouchDenial::UnknownGraphNode {
            graph_node_identity,
        })?
        .value();
    let transition = graph
        .mount_eligibility_transition_for_node(
            graph_node_identity,
            control_node
                .participation_posture()
                .axis(crate::graph::UiGraphParticipationAxis::Mounted),
            crate::graph::UiGraphAxisParticipation::runtime_mutation(
                crate::graph::UiGraphParticipationStatus::Admitted,
            ),
        )
        .ok_or(UiGraphTouchDenial::UnknownGraphNode {
            graph_node_identity,
        })?;
    let origin = graph.touches().query_binding_change_receipt()?;

    graph.touches().from_mount_eligibility_transition(
        origin,
        UiGraphTouchTiming::PostMutation,
        transition,
        UiGraphTouchAspects::new()
            .query_binding(UiGraphTouchAspectPosture::Invalidated)
            .participation(UiGraphTouchAspectPosture::Invalidated),
    )
}

pub(crate) fn try_allocation_touch_for_node(
    app: &WorthUiApp,
    graph_node_identity: UiGraphNodeIdentity,
) -> Result<UiGraphTouchDescriptor, UiGraphTouchDenial> {
    try_query_touch_for_node(app, graph_node_identity).or_else(|query_denial| {
        structural_touch_for_node(app, graph_node_identity).ok_or(query_denial)
    })
}

fn structural_touch_for_node(
    app: &WorthUiApp,
    graph_node_identity: UiGraphNodeIdentity,
) -> Option<UiGraphTouchDescriptor> {
    let artifact = declaration_artifact_for_graph_node(app, graph_node_identity)?;

    app.graph()
        .touches()
        .declaration_change_receipt(artifact)
        .ok()
        .and_then(|origin| {
            app.graph()
                .touches()
                .from_node(
                    origin,
                    UiGraphTouchTiming::PostMutation,
                    graph_node_identity,
                    UiGraphTouchAspects::new().structural(UiGraphTouchAspectPosture::Invalidated),
                )
                .ok()
        })
}
