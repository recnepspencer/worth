use super::super::WorthUiActiveApplicationSession;

use super::scroll_chrome_ingress::suppress_captured_axes;
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn observe_scroll_payload(
        &mut self,
        payload: &worth_ui_host_contract::UiHostObservationPayload,
        work: &mut crate::mounting::UiHitTestSpatialWork,
        observation_tick: Option<u64>,
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
                observation_tick,
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
        observation_tick: Option<u64>,
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
        let mut entries = Vec::with_capacity(chain.owners().len());
        for (slot, owner) in chain.owners().iter().copied().enumerate() {
            let incarnation = match owner {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => self
                    .scroll_region_incarnation(mounted_instance, slot)
                    .ok_or(UiHostScrollObservationDenial::AllocationUnavailable)?,
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => surface_incarnation,
            };
            entries.push(crate::runtime::scroll::UiScrollChainEntry::new(
                owner,
                incarnation,
            ));
        }
        // A thumb drag owns the axis it grabbed for the length of its capture.
        // The drag places that offset directly, so a wheel moving the same
        // offset underneath it would fight the pointer; the other axis of the
        // same region, and every other region, keep scrolling.
        let captured = self.axes_held_by_scroll_chrome(&entries);
        let delta_subpixels = suppress_captured_axes(delta_subpixels, captured);
        if captured != [false, false]
            && delta_subpixels == [0, 0]
            && [x_subpixels, y_subpixels] != [0, 0]
        {
            return Err(UiHostScrollObservationDenial::AxisHeldByChromeDrag);
        }
        let [x_subpixels, y_subpixels] = delta_subpixels;
        // A positive host delta moves content the way the reader pushed it; a
        // positive scroll offset moves content the other way. The sign turns
        // once, here, and every path below consumes offset-direction travel.
        let offset_delta = crate::runtime::scroll::UiScrollDelta::new(
            x_subpixels
                .checked_neg()
                .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)?,
            y_subpixels
                .checked_neg()
                .ok_or(UiHostScrollObservationDenial::DeltaOutOfRange)?,
        );
        // A declared smooth wheel answers a coarse notch with a settle
        // transition, so the observation itself moves no accepted offset: it
        // routes a zero delta to reconcile bounds and name the chain, then
        // stages the target the accepted sample travels to.
        let smooth = observation_tick.and_then(|tick| {
            self.admit_smooth_wheel_observation(
                mounted_instance,
                target.presentation(),
                phase,
                precision,
                offset_delta,
                tick,
            )
        });
        let delta = if smooth.is_some()
            || phase == worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled
        {
            crate::runtime::scroll::UiScrollDelta::new(0, 0)
        } else {
            offset_delta
        };
        let bounds = entries
            .iter()
            .enumerate()
            .map(|(slot, entry)| {
                self.scroll_bounds_for_mounted_owner(
                    entry.owner(),
                    mounted_instance,
                    mounted.graph_node_identity(),
                    slot,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_bounds_denial)?;
        let geometry = entries
            .iter()
            .enumerate()
            .map(|(slot, _)| {
                self.mounted
                    .scroll_region_geometry(mounted_instance, slot)
                    .map(|row| row.0)
            })
            .collect::<Vec<_>>();
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
        let mut successor = self
            .scroll
            .as_ref()
            .expect("Scroll installation was proven before bounds preflight")
            .clone();
        let receipt = successor
            .route_with_reconciled_bounds(request, &bounds)
            .map_err(UiHostScrollObservationDenial::Route)?;
        // `successor` is evidence until the pose it names is on mounted
        // geometry (or its settle is published). It becomes the session's
        // Scroll state only after that, so a refusal below leaves the accepted
        // offset exactly where the last applied pose left it.
        if let Some(observation) = smooth {
            // The staged settle lowers to a Motion transition request and is
            // published through the Scroll settle service-proposal lane. The
            // accepted offset does not move here: the track that settles it is
            // what moves it, one accepted sample at a time.
            let settled = self
                .stage_scroll_transition(&mut successor, &receipt, observation)
                .map_err(
                    |denial| crate::runtime::scroll::UiScrollSettleStop::Staging {
                        detail: format!("{denial:?}").into_boxed_str(),
                    },
                )
                .and_then(|transition| self.publish_scroll_settle(&transition));
            // The observation reports only that the settle went unpublished;
            // the reason is kept where a caller can read it back. An
            // unpublished settle also commits nothing: the staged target
            // travels with the discarded successor.
            match settled {
                Ok(()) => self.last_scroll_settle_stop = None,
                Err(stop) => {
                    self.last_scroll_settle_stop = Some(stop);
                    return Err(UiHostScrollObservationDenial::SettleUnpublished);
                }
            }
            *self.scroll.as_mut().expect("installed Scroll owner") = successor;
            return Ok(receipt);
        }
        let poses = receipt
            .transitions()
            .iter()
            .zip(geometry)
            .filter_map(|(transition, owner)| {
                owner.map(|owner| {
                    (
                        transition.owner().semantic_surface(),
                        owner,
                        transition.current(),
                    )
                })
            })
            .collect::<Vec<_>>();
        self.mounted
            .apply_scroll_geometries(&poses)
            .map_err(UiHostScrollObservationDenial::Geometry)?;
        *self.scroll.as_mut().expect("installed Scroll owner") = successor;
        Ok(receipt)
    }

    /// Which axes of this chain a thumb drag currently holds.
    ///
    /// The chain is the owner the delta would move and its ancestors, so a drag
    /// on any of them silences that axis for the whole chain: handing the
    /// remainder outward while the reader drags the thumb would scroll the
    /// ancestor instead, which is not what either gesture asked for.
    fn axes_held_by_scroll_chrome(
        &self,
        entries: &[crate::runtime::scroll::UiScrollChainEntry],
    ) -> [bool; 2] {
        use crate::runtime::scroll::chrome::UiScrollChromeAxis;

        let mut held = [false, false];
        for entry in entries {
            held[0] |= self.scroll_chrome_captures_axis(entry.owner(), UiScrollChromeAxis::Inline);
            held[1] |= self.scroll_chrome_captures_axis(entry.owner(), UiScrollChromeAxis::Block);
        }
        held
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
                // Content that travels with a Scroll region is laid out relative
                // to the region owner, not mounted beneath it, so a wheel over
                // that content addresses the region that scrolls it.
                let hit = target.view().mounted_instance();
                let mounted = self.mounted.scrolled_content_owner(hit).unwrap_or(hit);
                self.mounted
                    .current_mounted_identity_basis(mounted)
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
