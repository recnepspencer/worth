impl super::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_portal_motion_request(
        &self,
        transition: &crate::runtime::portal::UiPreparedPortalServiceTransition,
    ) -> Result<
        Option<crate::runtime::motion::UiMotionTransitionRequest>,
        crate::runtime::motion::UiMotionTransitionRequestDenial,
    > {
        if transition.is_idempotent() {
            return Ok(None);
        }
        let portal = transition.portal();
        let target = crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
            transition.request().semantic_surface(),
            portal.owner().mounted_instance_identity(),
            portal.diagnostic_value(),
        );
        // Named gate: a target whose retained exit is still settling physically
        // cannot accept a successor track, because committing one would displace
        // a retention its pending terminal still owns.
        if self
            .portal_exit_retention
            .physical_settlement_pending_for(target)
        {
            return Err(
                crate::runtime::motion::UiMotionTransitionRequestDenial::ExitRetentionAwaitingPhysicalSettlement,
            );
        }
        let predecessor = self
            .portal
            .as_ref()
            .and_then(|owner| owner.placement(portal))
            .map(|value| value.prepared());
        let successor = transition.placement();
        let presentation = predecessor
            .map(crate::runtime::portal::UiPreparedPortalPlacement::presentation)
            .or_else(|| {
                successor.map(crate::runtime::portal::UiPreparedPortalPlacement::presentation)
            })
            .expect("a non-idempotent portal transition retains current or successor placement");
        let committed_predecessor_geometry = predecessor
            .map(crate::runtime::portal::UiPreparedPortalPlacement::bounds)
            .map(crate::runtime::portal::UiPresentedPortalBounds::rect);
        let successor_geometry = successor
            .map(crate::runtime::portal::UiPreparedPortalPlacement::bounds)
            .map(crate::runtime::portal::UiPresentedPortalBounds::rect)
            .or(committed_predecessor_geometry);
        let predecessor_geometry = committed_predecessor_geometry.or_else(|| {
            transition
                .opens_portal()
                .then(|| successor_geometry.map(portal_entrance_start_geometry))
                .flatten()
        });
        // Reopening a Portal whose exit is still on screen is an entrance: the
        // exit faded opacity, so only an opacity-bearing successor can carry
        // the interrupted sample back to visible rather than snap it there.
        let reopens_exit = transition.opens_portal()
            && self
                .portal
                .as_ref()
                .is_some_and(|owner| owner.is_exiting(portal));
        let declaration = match (transition.opens_portal(), predecessor) {
            (true, Some(_)) if reopens_exit => {
                crate::runtime::motion::UiMotionDeclaration::portal_entrance()
            }
            (true, Some(_)) => crate::runtime::motion::UiMotionDeclaration::rebind_geometry(),
            (true, None) => crate::runtime::motion::UiMotionDeclaration::portal_entrance(),
            (false, _) => crate::runtime::motion::UiMotionDeclaration::portal_exit(),
        };
        let successor_presentation = successor
            .map(crate::runtime::portal::UiPreparedPortalPlacement::presentation)
            .unwrap_or(presentation);
        construct_portal_motion_transition(
            target,
            crate::runtime::motion::UiMotionTransitionEndpoint::new(
                transition.expected_revision(),
                presentation,
                predecessor_geometry,
                predecessor.is_some(),
            ),
            crate::runtime::motion::UiMotionTransitionEndpoint::new(
                transition.successor_revision(),
                successor_presentation,
                successor_geometry,
                transition.opens_portal(),
            ),
            declaration,
        )
        .map(Some)
    }
}

const PORTAL_ENTRANCE_TRANSLATION_Y: f32 = 8.0;

fn portal_entrance_start_geometry(
    successor: crate::mounting::presentation::UiPublishedRect,
) -> crate::mounting::presentation::UiPublishedRect {
    successor.translated([0.0, PORTAL_ENTRANCE_TRANSLATION_Y])
}

/// A Portal transition whose successor keeps the predecessor's presentation
/// family is an ordinary family transition; one whose successor is published
/// against another binding or host surface is a rebind, and is constructed as
/// one so the changed binding is admitted rather than refused.
fn construct_portal_motion_transition(
    target: crate::runtime::motion::UiMotionTargetIdentity,
    predecessor: crate::runtime::motion::UiMotionTransitionEndpoint,
    successor: crate::runtime::motion::UiMotionTransitionEndpoint,
    declaration: crate::runtime::motion::UiMotionDeclaration,
) -> Result<
    crate::runtime::motion::UiMotionTransitionRequest,
    crate::runtime::motion::UiMotionTransitionRequestDenial,
> {
    let constructor = if predecessor.presentation().binding() == successor.presentation().binding()
        && predecessor.presentation().host_surface() == successor.presentation().host_surface()
    {
        crate::runtime::motion::UiMotionTransitionRequest::from_family_transition
    } else {
        crate::runtime::motion::UiMotionTransitionRequest::from_rebind_transition
    };
    constructor(target, predecessor, successor, declaration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_entrance_start_is_explicit_and_preserves_viewport_brand() {
        let successor = crate::mounting::presentation::UiPublishedRect::from_committed_components(
            [12.0, 20.0, 40.0, 24.0],
            worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        )
        .unwrap();
        let predecessor = portal_entrance_start_geometry(successor);

        assert_eq!(predecessor.components(), [12.0, 28.0, 40.0, 24.0]);
        assert_eq!(
            predecessor.coordinate_space(),
            worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        );
    }

    #[test]
    fn portal_proposal_compilation_uses_the_rebind_constructor_for_a_successor_binding() {
        let semantic = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let mounted = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let host = worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap();
        let predecessor = presentation(host, 1);
        let successor = presentation(host, 2);
        let geometry = crate::mounting::presentation::UiPublishedRect::from_committed_components(
            [12.0, 20.0, 40.0, 24.0],
            worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        )
        .unwrap();

        assert!(matches!(
            crate::runtime::motion::UiMotionTransitionRequest::from_family_transition(
                crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
                    semantic, mounted, 7,
                ),
                crate::runtime::motion::UiMotionTransitionEndpoint::new(
                    1,
                    predecessor,
                    Some(geometry),
                    true,
                ),
                crate::runtime::motion::UiMotionTransitionEndpoint::new(
                    2,
                    successor,
                    Some(geometry),
                    true,
                ),
                crate::runtime::motion::UiMotionDeclaration::rebind_geometry(),
            ),
            Err(crate::runtime::motion::UiMotionTransitionRequestDenial::BindingChangedWithoutRebind)
        ));
        construct_portal_motion_transition(
            crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(semantic, mounted, 7),
            crate::runtime::motion::UiMotionTransitionEndpoint::new(
                1,
                predecessor,
                Some(geometry),
                true,
            ),
            crate::runtime::motion::UiMotionTransitionEndpoint::new(
                2,
                successor,
                Some(geometry),
                true,
            ),
            crate::runtime::motion::UiMotionDeclaration::rebind_geometry(),
        )
        .expect("production portal proposal compilation must admit the rebind successor");
    }

    fn presentation(
        host: worth_ui_host_contract::UiHostSurfaceIdentity,
        epoch: u64,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        worth_ui_host_contract::UiHostObservationPresentationBasis::new(
            host,
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(epoch),
        )
    }
}
