//! SUPPORT AUTHORITY for component-level mounted-frame certification.

use crate::facade::{WorthUiActiveApplicationSession, WorthUiActiveFrameworkTurnExecution};

pub trait WorthUiMountedFrameExecutionCertificationExt {
    fn classify_mounted_frame_reuse(
        &self,
        request: &crate::mounting::UiMountedFrameRequest,
    ) -> crate::mounting::UiMountedFrameReuse;

    fn prepare_mounted_frame(
        self,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    >;
}

pub trait WorthUiMountedPublicationCertificationExt {
    fn present_prepared_mounted_frame(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> crate::mounting::UiMountedFrameOutcome;

    fn acquire_visual_overlay_lease(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<
        UiMountedVisualOverlayLeaseCertificationReceipt,
        crate::mounting::UiMountedRetentionClass,
    >;
}

pub struct UiMountedVisualOverlayLeaseCertificationReceipt {
    lease: crate::mounting::UiMountedVisualOverlayLease,
}

impl<'session> WorthUiMountedFrameExecutionCertificationExt
    for WorthUiActiveFrameworkTurnExecution<'session>
{
    fn classify_mounted_frame_reuse(
        &self,
        request: &crate::mounting::UiMountedFrameRequest,
    ) -> crate::mounting::UiMountedFrameReuse {
        self.classify_mounted_frame_reuse_internal(request)
    }

    fn prepare_mounted_frame(
        self,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let mut execution = self;
        execution.prepare_mounted_frame_internal(request)
    }
}

impl WorthUiMountedPublicationCertificationExt for WorthUiActiveApplicationSession {
    fn present_prepared_mounted_frame(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> crate::mounting::UiMountedFrameOutcome {
        self.present_prepared_mounted_frame_internal(frame, deadline, now)
    }

    fn acquire_visual_overlay_lease(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Result<
        UiMountedVisualOverlayLeaseCertificationReceipt,
        crate::mounting::UiMountedRetentionClass,
    > {
        self.acquire_visual_overlay_for_certification(frame, binding)
            .map(|lease| UiMountedVisualOverlayLeaseCertificationReceipt { lease })
            .map_err(visual_retention_denial_class)
    }
}

impl UiMountedVisualOverlayLeaseCertificationReceipt {
    pub fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.lease.frame()
    }

    pub fn structural_bytes(&self) -> usize {
        self.lease.structural_bytes()
    }
}

fn visual_retention_denial_class(
    denial: crate::mounting::UiMountedVisualRetentionDenial,
) -> crate::mounting::UiMountedRetentionClass {
    match denial {
        crate::mounting::UiMountedVisualRetentionDenial::CapacityExceeded { class, .. }
        | crate::mounting::UiMountedVisualRetentionDenial::AccountingOverflow { class } => class,
        crate::mounting::UiMountedVisualRetentionDenial::ExpiredFrame
        | crate::mounting::UiMountedVisualRetentionDenial::UnknownFrame => {
            crate::mounting::UiMountedRetentionClass::VisualOverlay
        }
    }
}
