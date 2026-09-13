use super::super::WorthUiActiveApplicationSession;

use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn observe_scroll_payload(
        &mut self,
        payload: &worth_ui_host_contract::UiHostObservationPayload,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Option<UiHostScrollObservationOutcome> {
        let worth_ui_host_contract::UiHostObservationPayload::ScrollDelta {
            source,
            phase,
            precision,
            target,
            x_subpixels,
            y_subpixels,
        } = payload
        else {
            return None;
        };
        Some(
            match self.apply_host_scroll_delta(
                *source,
                *phase,
                *precision,
                *target,
                [*x_subpixels, *y_subpixels],
                work,
            ) {
                Ok(receipt) => UiHostScrollObservationOutcome::Applied(receipt),
                Err(denial) => UiHostScrollObservationOutcome::Denied(denial),
            },
        )
    }

    fn apply_host_scroll_delta(
        &mut self,
        source: worth_ui_host_contract::UiHostScrollDeltaSource,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
        target: worth_ui_host_contract::UiHostScrollDeltaTargetAffinity,
        delta_subpixels: [i64; 2],
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<crate::runtime::scroll::UiScrollRouteReceipt, UiHostScrollObservationDenial> {
        let (mounted_instance, mounted) = self.resolve_scroll_target(target, work)?;
        let [x_subpixels, y_subpixels] = delta_subpixels;
        let surface_incarnation = self.scroll_owner_incarnation();
        let scroll = self
            .scroll
            .as_ref()
            .ok_or(UiHostScrollObservationDenial::NoDeclaredScrollOwner)?;
        let chain = scroll
            .ownership_chain(mounted_instance)
            .map_err(UiHostScrollObservationDenial::Ownership)?;
        if chain.owners().is_empty() {
            return Err(UiHostScrollObservationDenial::NoDeclaredScrollOwner);
        }
        let mounted_incarnation =
            crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                mounted.mount_incarnation(),
            );
        let mut entries = Vec::with_capacity(chain.owners().len());
        for owner in chain.owners().iter().copied() {
            let incarnation = match owner {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => mounted_incarnation,
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => surface_incarnation,
            };
            entries.push(crate::runtime::scroll::UiScrollChainEntry::new(
                owner,
                incarnation,
            ));
        }
        let delta = if phase == worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled {
            crate::runtime::scroll::UiScrollDelta::new(0, 0)
        } else {
            crate::runtime::scroll::UiScrollDelta::new(
                x_subpixels
                    .checked_neg()
                    .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)?,
                y_subpixels
                    .checked_neg()
                    .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)?,
            )
        };
        let bounds = self
            .application
            .scroll_bounds_for_chain(&entries, mounted.graph_node_identity())
            .map_err(map_bounds_denial)?;
        let request = crate::runtime::scroll::UiScrollDeltaRequest::new(
            entries,
            delta,
            crate::runtime::scroll::UiScrollDeltaCause::Host {
                source,
                phase,
                precision,
            },
        )
        .map_err(UiHostScrollObservationDenial::Route)?;
        self.scroll
            .as_mut()
            .expect("Scroll installation was proven before bounds preflight")
            .route_with_reconciled_bounds(request, &bounds)
            .map_err(UiHostScrollObservationDenial::Route)
    }

    fn resolve_scroll_target(
        &self,
        target: worth_ui_host_contract::UiHostScrollDeltaTargetAffinity,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<
        (
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::UiMountedIdentityBasis,
        ),
        UiHostScrollObservationDenial,
    > {
        match target {
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::ExactCoordinate {
                presentation,
                position,
            } => {
                crate::runtime::interaction::targeting::require_current_presentation(
                    &self.mounted,
                    presentation,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                let target = crate::runtime::interaction::targeting::resolve_presented_target(
                    &self.mounted,
                    presentation,
                    position,
                    work,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                let mounted = target.view().mounted_instance();
                self.mounted
                    .current_mounted_identity_basis(target.view().mounted_instance())
                    .map(|basis| (mounted, basis))
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)
            }
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::ExactMountedTarget {
                presentation,
                mounted,
            } => {
                crate::runtime::interaction::targeting::require_current_presentation(
                    &self.mounted,
                    presentation,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                let surface = self
                    .mounted
                    .current_surface_for_binding(presentation.binding())
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)?;
                self.mounted
                    .admit_current_interaction_affinity(
                        crate::mounting::UiMountedInteractionAffinityInput {
                            surface,
                            binding: presentation.binding(),
                            mounted_instance: mounted.instance(),
                            node_receipt: mounted.node_receipt(),
                        },
                    )
                    .map_err(|denial| {
                        UiHostScrollObservationDenial::Targeting(
                            crate::runtime::interaction::targeting::map_current_affinity_denial(
                                denial,
                            ),
                        )
                    })?;
                self.mounted
                    .current_mounted_identity_basis(mounted.instance())
                    .map(|basis| (mounted.instance(), basis))
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)
            }
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::PresentedSurfaceFallback {
                ..
            } => Err(UiHostScrollObservationDenial::PresentedSurfaceFallbackIsAmbiguous),
        }
    }
}

fn map_bounds_denial(
    denial: crate::runtime::scroll::UiScrollBoundsResolutionDenial,
) -> UiHostScrollObservationDenial {
    match denial {
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable => {
            UiHostScrollObservationDenial::AllocationUnavailable
        }
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::ViewportUnavailable => {
            UiHostScrollObservationDenial::ViewportUnavailable
        }
        crate::runtime::scroll::UiScrollBoundsResolutionDenial::OutOfRange => {
            UiHostScrollObservationDenial::BoundsOutOfRange
        }
    }
}
