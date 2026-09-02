pub use crate::host_exchange::observation_report_validation::{
    UiDuplicateHostObservationBatch, UiHostObservationBatchDisposition, UiHostObservationCapacity,
    UiHostObservationCapacityInput, UiHostObservationDisposition, UiHostObservationFrameRelation,
    UiHostObservationReportDenial, UiHostObservationReportOutcome, UiHostObservationWorkReport,
    UiQuarantinedHostObservationBatch, UiValidatedHostObservationBatch,
    UiValidatedHostObservationReport,
};
pub use worth_ui_host_contract::{
    UiHostImeCompositionPhase, UiHostImePreedit, UiHostImePreeditConstructionDenial,
    UiHostImePreeditSelection, UiHostImeRangeConversionReceipt, UiHostKey, UiHostKeyTransition,
    UiHostKeyboardModifiers, UiHostObservationBatch, UiHostObservationBatchConstructionDenial,
    UiHostObservationBatchInput, UiHostObservationCanonicalCore,
    UiHostObservationCanonicalCoreInput, UiHostObservationCoalescingIdentity,
    UiHostObservationDrain, UiHostObservationDrainDenial, UiHostObservationFamily,
    UiHostObservationIntegrity, UiHostObservationLoss, UiHostObservationMountedBasis,
    UiHostObservationPayload, UiHostObservationPointerDeviceKindDenial,
    UiHostObservationPresentationBasis, UiHostObservationReport,
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
    UiHostPointerButton, UiHostPointerButtonTransition, UiHostPointerCaptureEpoch,
    UiHostPointerDeviceKind, UiHostPointerIdentity, UiHostPressedPointerButtons,
    UiHostSurfaceCoordinateSpace,
    UiHostSurfaceCoordinateUnit, UiHostSurfacePosition, UiHostSurfacePositionBasis,
    UiHostUnicodeScalarRange, UiHostUtf8ByteRange, UI_HOST_OBSERVATION_BATCH_BYTE_LIMIT,
    UI_HOST_OBSERVATION_BATCH_REPORT_LIMIT, UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT,
    UI_HOST_OBSERVATION_DRAIN_BYTE_LIMIT, UI_HOST_OBSERVATION_DRAIN_REPORT_LIMIT,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
pub use worth_ui_host_contract::{
    UiHostProtocolAgreement, UiHostProtocolContract, UiHostProtocolNegotiation,
};

/// Host-integrator access to adapter drain and structural validation.
pub trait WorthUiHostObservationSessionExt {
    fn validate_host_observation_batch(
        &mut self,
        batch: UiHostObservationBatch,
    ) -> UiHostObservationReportOutcome;

    fn retained_host_observation_report_count(&self) -> usize;

    fn retained_host_observation_byte_count(&self) -> usize;

    fn quarantined_host_observation_batch_count(&self) -> usize;

    fn quarantined_host_observation_byte_count(&self) -> usize;

    fn host_observation_work_report(&self) -> UiHostObservationWorkReport;

    fn drain_and_validate_host_observation_batches(
        &mut self,
    ) -> Result<Box<[UiHostObservationReportOutcome]>, UiHostObservationDrainDenial>;
}

impl WorthUiHostObservationSessionExt for crate::facade::WorthUiActiveApplicationSession {
    fn validate_host_observation_batch(
        &mut self,
        batch: UiHostObservationBatch,
    ) -> UiHostObservationReportOutcome {
        crate::facade::WorthUiActiveApplicationSession::validate_host_observation_batch(self, batch)
    }

    fn retained_host_observation_report_count(&self) -> usize {
        crate::facade::WorthUiActiveApplicationSession::retained_host_observation_report_count(self)
    }

    fn retained_host_observation_byte_count(&self) -> usize {
        crate::facade::WorthUiActiveApplicationSession::retained_host_observation_byte_count(self)
    }

    fn quarantined_host_observation_batch_count(&self) -> usize {
        crate::facade::WorthUiActiveApplicationSession::quarantined_host_observation_batch_count(
            self,
        )
    }

    fn quarantined_host_observation_byte_count(&self) -> usize {
        crate::facade::WorthUiActiveApplicationSession::quarantined_host_observation_byte_count(
            self,
        )
    }

    fn host_observation_work_report(&self) -> UiHostObservationWorkReport {
        crate::facade::WorthUiActiveApplicationSession::host_observation_work_report(self)
    }

    fn drain_and_validate_host_observation_batches(
        &mut self,
    ) -> Result<Box<[UiHostObservationReportOutcome]>, UiHostObservationDrainDenial> {
        crate::facade::WorthUiActiveApplicationSession::drain_and_validate_host_observation_batches(
            self,
        )
    }
}
