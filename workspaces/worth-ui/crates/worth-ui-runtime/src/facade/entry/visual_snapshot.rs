use worth_ui_inspection::{
    SealedPixelArtifactPolicy, UiGeometryOnly, UiVisualArtifactPolicy, UiVisualSnapshotDenial,
    UiVisualSnapshotRequest,
};

use super::{WorthUiActiveApplicationSession, WorthUiNativeApplicationShell};
use crate::inspection::visual_snapshot::{
    UiPendingVisualCapture, UiVisualCancellationReceipt, UiVisualCaptureIntent,
    UiVisualCapturePoll, UiVisualGeometryGrant, UiVisualPixelCaptureGrant,
    UiVisualSnapshotComparisonGrant, UiVisualSnapshotComparisonRequest, UiVisualTarget,
    WorthUiVisualInspectionAuthority,
};

mod capture_poll;
mod coordinate_projection;

impl WorthUiActiveApplicationSession {
    pub fn visual_inspection_authority(&self) -> &WorthUiVisualInspectionAuthority {
        &self.visual_inspection
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn acquire_visual_overlay_for_certification(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<
        crate::mounting::UiMountedVisualOverlayLease,
        crate::mounting::UiMountedVisualRetentionDenial,
    > {
        self.mounted
            .acquire_visual_overlay_for_certification(frame, binding)
    }

    pub fn begin_visual_geometry_snapshot<Target: UiVisualTarget>(
        &mut self,
        grant: &UiVisualGeometryGrant,
        request: UiVisualSnapshotRequest<Target, UiGeometryOnly>,
    ) -> Result<UiPendingVisualCapture<Target, UiGeometryOnly>, UiVisualSnapshotDenial> {
        self.admit_visual_capture(grant.session(), grant.scope(), request)
    }

    pub fn begin_visual_pixel_snapshot<Target, Policy>(
        &mut self,
        grant: &UiVisualPixelCaptureGrant,
        request: UiVisualSnapshotRequest<Target, Policy>,
    ) -> Result<UiPendingVisualCapture<Target, Policy>, UiVisualSnapshotDenial>
    where
        Target: UiVisualTarget,
        Policy: SealedPixelArtifactPolicy,
    {
        self.admit_visual_capture(grant.session(), grant.scope(), request)
    }

    fn admit_visual_capture<Target, Policy>(
        &mut self,
        grant_session: super::WorthUiActiveApplicationSessionIdentity,
        grant_scope: crate::inspection::visual_snapshot::UiVisualGrantScope,
        request: UiVisualSnapshotRequest<Target, Policy>,
    ) -> Result<UiPendingVisualCapture<Target, Policy>, UiVisualSnapshotDenial>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        if grant_session != self.identity {
            return Err(UiVisualSnapshotDenial::ForeignSession);
        }
        if grant_scope.audience() != self.visual_inspection.policy().audience() {
            return Err(UiVisualSnapshotDenial::Disclosure);
        }
        if grant_scope.disclosure() != request.disclosure() {
            return Err(UiVisualSnapshotDenial::Disclosure);
        }
        let (intent, target) = UiVisualCaptureIntent::from_request(request);
        let route = crate::inspection::visual_snapshot::into_capture_route(target);
        match route {
            crate::inspection::visual_snapshot::UiVisualTargetRoute::Host(basis) => {
                self.admit_host_visual_capture(grant_scope, intent, basis)
            }
            crate::inspection::visual_snapshot::UiVisualTargetRoute::DerivedRegion(source) => {
                self.admit_derived_visual_capture(grant_scope, intent, source)
            }
        }
    }

    fn reserve_visual_capture_identity(&mut self) -> Result<u64, UiVisualSnapshotDenial> {
        let identity = self.next_visual_capture_identity;
        self.next_visual_capture_identity = identity
            .checked_add(1)
            .ok_or(UiVisualSnapshotDenial::CapacityExceeded)?;
        Ok(identity)
    }

    fn admit_host_visual_capture<Target, Policy>(
        &mut self,
        grant_scope: crate::inspection::visual_snapshot::UiVisualGrantScope,
        intent: UiVisualCaptureIntent<Target, Policy>,
        basis: crate::inspection::visual_snapshot::UiVisualSurfaceCaptureBasis,
    ) -> Result<UiPendingVisualCapture<Target, Policy>, UiVisualSnapshotDenial>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        let mounted_capture_basis = self
            .mounted
            .acquire_visual_snapshot(basis.frame, basis.binding)
            .map_err(map_visual_retention_denial)?;
        let (snapshot_lease, visual_regions, identity_trace_basis) =
            mounted_capture_basis.into_parts();
        let structural_bytes =
            crate::inspection::visual_snapshot::structural_reservation::host_structural_reservation(
                grant_scope,
                &snapshot_lease,
                &visual_regions,
                &identity_trace_basis,
            )?;
        let reservation = crate::inspection::visual_snapshot::UiVisualResourceReservation::new(
            reserved_host_pixel_bytes(Policy::PIXELS_REQUESTED, grant_scope),
            structural_bytes,
        );
        let identity = self.reserve_visual_capture_identity()?;
        let registration = self
            .visual_captures
            .register(identity, basis.host_surface, reservation)
            .map_err(map_visual_registration_denial)?;
        let pinned = intent.admit().pin(
            crate::inspection::visual_snapshot::UiPinnedVisualCaptureInput {
                session: self.identity,
                capture_identity: identity,
                presentation: basis,
                snapshot_lease,
                visual_regions,
                identity_trace_basis,
                registration,
            },
        );
        Ok(UiPendingVisualCapture::pinned(pinned))
    }

    fn admit_derived_visual_capture<Target, Policy>(
        &mut self,
        grant_scope: crate::inspection::visual_snapshot::UiVisualGrantScope,
        intent: UiVisualCaptureIntent<Target, Policy>,
        source: crate::inspection::visual_snapshot::UiDerivedRegionTargetSource,
    ) -> Result<UiPendingVisualCapture<Target, Policy>, UiVisualSnapshotDenial>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        validate_derived_pixel_capacity(Policy::PIXELS_REQUESTED, grant_scope, source.region)?;
        if source.snapshot.evidence.cost().retained_structural_bytes()
            > grant_scope.maximum_retained_structural_bytes_per_receipt()
        {
            return Err(UiVisualSnapshotDenial::RetainedStructurePerReceiptCapacityExceeded);
        }
        let identity = self.reserve_visual_capture_identity()?;
        Ok(UiPendingVisualCapture::derived_region(
            crate::inspection::visual_snapshot::UiPendingDerivedRegionInput {
                capture_identity: identity,
                deadline: intent.capture_deadline(),
                source: source.snapshot,
                region: source.region,
            },
        ))
    }

    pub fn cancel_visual_snapshot<Target, Policy>(
        &mut self,
        pending: UiPendingVisualCapture<Target, Policy>,
    ) -> UiVisualCancellationReceipt
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        let Some(request) = pending.host_request() else {
            return pending.cancel_before_host();
        };
        let host = self.host_session.effect_port();
        let posture = match host
            .adapter()
            .cancel_visual_capture(host.authority(), request)
        {
            worth_ui_host_contract::UiHostCaptureCancellationOutcome::CancelledBeforeReadback => {
                crate::inspection::visual_snapshot::UiVisualCancellationPosture::CancelledBeforeReadback
            }
            worth_ui_host_contract::UiHostCaptureCancellationOutcome::ReadbackMayHaveBegun => {
                crate::inspection::visual_snapshot::UiVisualCancellationPosture::ReadbackMayHaveBegun
            }
            worth_ui_host_contract::UiHostCaptureCancellationOutcome::CleanupIndeterminate => {
                crate::inspection::visual_snapshot::UiVisualCancellationPosture::CleanupIndeterminate
            }
        };
        pending.cancel(posture)
    }

    pub fn dispose_visual_snapshot<Posture>(
        &mut self,
        receipt: crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Posture>,
    ) -> crate::inspection::visual_snapshot::UiVisualSnapshotDisposalReceipt
    where
        Posture: UiVisualArtifactPolicy,
    {
        receipt.dispose()
    }

    pub fn compare_visual_snapshots<Predecessor, Successor>(
        &self,
        grant: &UiVisualSnapshotComparisonGrant,
        request: UiVisualSnapshotComparisonRequest<'_, Predecessor, Successor>,
    ) -> worth_ui_inspection::UiVisualSnapshotComparisonOutcome
    where
        Predecessor: UiVisualArtifactPolicy,
        Successor: UiVisualArtifactPolicy,
    {
        crate::inspection::visual_snapshot::compare_visual_snapshots(
            self.identity,
            grant,
            request,
            || reserve_visual_comparison(&self.rebind),
        )
    }
}

fn reserve_visual_comparison(
    state: &crate::runtime::rebind::UiRebindRuntimeState,
) -> Result<
    crate::runtime::rebind::UiRebindComparisonReservation,
    worth_ui_inspection::UiVisualSnapshotComparisonOutcome,
> {
    use worth_ui_inspection::UiVisualSnapshotComparisonOutcome as Outcome;
    match state.reserve_comparison_snapshots(2) {
        Ok(reservation) => Ok(reservation),
        Err(
            crate::runtime::rebind::UiRebindReservationDenial::
                RetainedComparisonSnapshotCapacityExceeded {
                    configured,
                    required,
                },
        ) => Err(Outcome::Denied(
            worth_ui_inspection::UiVisualSnapshotComparisonDenial::retained_snapshot_capacity(
                configured, required,
            ),
        )),
        Err(crate::runtime::rebind::UiRebindReservationDenial::AdmissionClosed) => Err(
            Outcome::Expired(worth_ui_inspection::UiVisualSnapshotComparisonExpiry::Both),
        ),
        Err(_) => unreachable!("comparison reservation uses only its dedicated registry lane"),
    }
}

#[inline(never)]
fn reserved_host_pixel_bytes(
    pixels_requested: bool,
    grant_scope: crate::inspection::visual_snapshot::UiVisualGrantScope,
) -> u64 {
    if !pixels_requested {
        return 0;
    }
    grant_scope
        .maximum_capture_bytes()
        .min(grant_scope.maximum_retained_pixel_bytes())
}

#[inline(never)]
fn validate_derived_pixel_capacity(
    pixels_requested: bool,
    grant_scope: crate::inspection::visual_snapshot::UiVisualGrantScope,
    region: worth_ui_inspection::UiClientPhysicalRect,
) -> Result<(), UiVisualSnapshotDenial> {
    if !pixels_requested {
        return Ok(());
    }
    let width = u64::from(region.right() - region.left());
    let height = u64::from(region.bottom() - region.top());
    let required = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(UiVisualSnapshotDenial::CapacityExceeded)?;
    if required > reserved_host_pixel_bytes(pixels_requested, grant_scope) {
        return Err(UiVisualSnapshotDenial::RetainedPixelCapacityExceeded);
    }
    Ok(())
}

fn map_visual_retention_denial(
    denial: crate::mounting::UiMountedVisualRetentionDenial,
) -> UiVisualSnapshotDenial {
    match denial {
        crate::mounting::UiMountedVisualRetentionDenial::CapacityExceeded { .. }
        | crate::mounting::UiMountedVisualRetentionDenial::AccountingOverflow { .. } => {
            UiVisualSnapshotDenial::CapacityExceeded
        }
        crate::mounting::UiMountedVisualRetentionDenial::ExpiredFrame
        | crate::mounting::UiMountedVisualRetentionDenial::UnknownFrame => {
            panic!("a live mounted visual target must preserve its retained frame")
        }
    }
}

fn map_visual_registration_denial(
    denial: crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial,
) -> UiVisualSnapshotDenial {
    match denial {
        crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::SnapshotCapacityExceeded => {
            UiVisualSnapshotDenial::SnapshotCapacityExceeded
        }
        crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::PixelRetentionCapacityExceeded => {
            UiVisualSnapshotDenial::RetainedPixelCapacityExceeded
        }
        crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::StructuralRetentionCapacityExceeded => {
            UiVisualSnapshotDenial::RetainedStructurePerSessionCapacityExceeded
        }
        crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::RegistryClosed
        | crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::SurfaceCaptureInFlight
        | crate::inspection::visual_snapshot::UiVisualCaptureRegistrationDenial::AccountingOverflow => {
            UiVisualSnapshotDenial::CapacityExceeded
        }
    }
}

impl WorthUiNativeApplicationShell {
    pub fn visual_inspection_authority(&self) -> &WorthUiVisualInspectionAuthority {
        self.session.visual_inspection_authority()
    }

    pub fn begin_visual_geometry_snapshot<Target: UiVisualTarget>(
        &mut self,
        grant: &UiVisualGeometryGrant,
        request: UiVisualSnapshotRequest<Target, UiGeometryOnly>,
    ) -> Result<UiPendingVisualCapture<Target, UiGeometryOnly>, UiVisualSnapshotDenial> {
        self.session.begin_visual_geometry_snapshot(grant, request)
    }

    pub fn begin_visual_pixel_snapshot<Target, Policy>(
        &mut self,
        grant: &UiVisualPixelCaptureGrant,
        request: UiVisualSnapshotRequest<Target, Policy>,
    ) -> Result<UiPendingVisualCapture<Target, Policy>, UiVisualSnapshotDenial>
    where
        Target: UiVisualTarget,
        Policy: SealedPixelArtifactPolicy,
    {
        self.session.begin_visual_pixel_snapshot(grant, request)
    }

    pub fn poll_visual_snapshot<Target, Policy>(
        &mut self,
        pending: UiPendingVisualCapture<Target, Policy>,
        now_tick: u64,
    ) -> UiVisualCapturePoll<Target, Policy>
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        self.session.poll_visual_snapshot(pending, now_tick)
    }

    pub fn cancel_visual_snapshot<Target, Policy>(
        &mut self,
        pending: UiPendingVisualCapture<Target, Policy>,
    ) -> UiVisualCancellationReceipt
    where
        Target: UiVisualTarget,
        Policy: UiVisualArtifactPolicy,
    {
        self.session.cancel_visual_snapshot(pending)
    }

    pub fn dispose_visual_snapshot<Posture>(
        &mut self,
        receipt: crate::inspection::visual_snapshot::UiVisualSnapshotReceipt<Posture>,
    ) -> crate::inspection::visual_snapshot::UiVisualSnapshotDisposalReceipt
    where
        Posture: UiVisualArtifactPolicy,
    {
        self.session.dispose_visual_snapshot(receipt)
    }

    pub fn compare_visual_snapshots<Predecessor, Successor>(
        &self,
        grant: &UiVisualSnapshotComparisonGrant,
        request: UiVisualSnapshotComparisonRequest<'_, Predecessor, Successor>,
    ) -> worth_ui_inspection::UiVisualSnapshotComparisonOutcome
    where
        Predecessor: UiVisualArtifactPolicy,
        Successor: UiVisualArtifactPolicy,
    {
        self.session.compare_visual_snapshots(grant, request)
    }
}
