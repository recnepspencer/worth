pub(super) fn portal_service_request(
    handoff: &crate::runtime::intent_execution::UiIntentConsequenceHandoff,
    destination: crate::capability::UiIntentRuntimeServiceDestination,
    presented_viewport: Option<crate::runtime::interaction::UiPresentedViewportGeometry>,
    resolved_owner: Option<(
        crate::graph::UiGraphNodeIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )>,
) -> Result<
    crate::runtime::portal::UiPortalServiceRequest,
    crate::runtime::portal::UiPortalPlacementDenial,
> {
    use crate::runtime::portal::{UiPortalIdentity, UiPortalOwnerIdentity, UiPortalServiceRequest};
    let target = UiPortalIdentity::for_owner(UiPortalOwnerIdentity::from_target(
        handoff.graph_node(),
        handoff.target(),
    ));
    let containing_portal = resolved_owner.map(|(graph_node, mounted_instance)| {
        UiPortalIdentity::for_owner(UiPortalOwnerIdentity::from_mounted_owner(
            graph_node,
            mounted_instance,
        ))
    });
    let request = match destination {
        crate::capability::UiIntentRuntimeServiceDestination::OpenPortal => match containing_portal
        {
            Some(parent) => UiPortalServiceRequest::open_nested(
                target,
                handoff.idempotency(),
                handoff.target().geometry(),
                presented_viewport.ok_or(
                    crate::runtime::portal::UiPortalPlacementDenial::MissingPresentedViewport,
                )?,
                handoff.target().surface(),
                parent,
                crate::runtime::portal::UiPortalInputShielding::ContentBounds,
            ),
            None => UiPortalServiceRequest::open(
                target,
                handoff.idempotency(),
                handoff.target().geometry(),
                presented_viewport,
                handoff.target().surface(),
            ),
        },
        crate::capability::UiIntentRuntimeServiceDestination::ClosePortal => {
            crate::runtime::portal::UiPortalServiceRequest::close(
                containing_portal.unwrap_or(target),
                handoff.idempotency(),
                crate::runtime::portal::UiPortalDismissalCause::ExplicitOwnerRequest,
                handoff.target().surface(),
            )
        }
        crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand => {
            unreachable!("command consequences never construct mounted portal service requests")
        }
    };
    Ok(match destination {
        crate::capability::UiIntentRuntimeServiceDestination::OpenPortal => {
            request.with_declared_portal(handoff.authored_portal_declaration())
        }
        crate::capability::UiIntentRuntimeServiceDestination::ClosePortal
        | crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand => request,
    })
}

pub(super) fn portal_placement_stop_reason(
    denial: crate::runtime::portal::UiPortalPlacementDenial,
) -> crate::runtime::intent_execution::UiIntentPortalPlacementStopReason {
    use crate::runtime::intent_execution::UiIntentPortalPlacementStopReason as Stop;
    match denial {
        crate::runtime::portal::UiPortalPlacementDenial::MissingPresentedAnchor => {
            Stop::MissingPresentedAnchor
        }
        crate::runtime::portal::UiPortalPlacementDenial::MissingPresentedViewport => {
            Stop::MissingPresentedViewport
        }
        crate::runtime::portal::UiPortalPlacementDenial::IncompatibleCoordinateSpace => {
            Stop::IncompatibleCoordinateSpace
        }
        crate::runtime::portal::UiPortalPlacementDenial::EmptyAnchor => Stop::EmptyAnchor,
        crate::runtime::portal::UiPortalPlacementDenial::InsufficientViewport => {
            Stop::InsufficientViewport
        }
        crate::runtime::portal::UiPortalPlacementDenial::UnknownParent => Stop::UnknownPortalParent,
        crate::runtime::portal::UiPortalPlacementDenial::LayerDepthExhausted => {
            Stop::PortalLayerDepthExhausted
        }
    }
}
