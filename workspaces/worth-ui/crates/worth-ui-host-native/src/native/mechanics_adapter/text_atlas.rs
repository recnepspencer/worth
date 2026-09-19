//! Native adapter bridge for the borrowed, scoped text-atlas transaction.

use crate::native::UiNativeHostState;
use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterMissRasterizer, UiGlyphRasterPinTransitionView,
    UiGlyphRasterTransactionOutcome,
};

use self::text_atlas_upload::RealTextAtlasUploadPort;
#[cfg(test)]
use self::text_atlas_upload::UiNativeTextAtlasUploadPort;

pub(super) fn perform(
    state: &mut UiNativeHostState,
    presentation_basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
    demands: &[UiGlyphRasterDemandBatchView<'_>],
    pins: UiGlyphRasterPinTransitionView<'_>,
    rasterizer: &mut dyn UiGlyphRasterMissRasterizer,
) -> UiGlyphRasterTransactionOutcome {
    let mut upload_port = RealTextAtlasUploadPort;
    text_atlas_transaction::perform(text_atlas_transaction::TextAtlasExecution {
        state,
        presentation_basis,
        demands,
        pins,
        rasterizer,
        upload_port: &mut upload_port,
    })
}

pub(super) fn release_pins(
    state: &mut UiNativeHostState,
    request: worth_ui_host_contract::UiMountedTextPinReleaseRequest,
    pins: UiGlyphRasterPinTransitionView<'_>,
) -> UiGlyphRasterTransactionOutcome {
    struct NoMissRasterizer;

    impl UiGlyphRasterMissRasterizer for NoMissRasterizer {
        fn rasterize(
            &mut self,
            _misses: worth_ui_host_contract::UiGlyphRasterMissSelectionView<'_>,
            _sink: &mut dyn worth_ui_host_contract::UiGlyphRasterBatchSink,
        ) -> Result<(), worth_ui_host_contract::UiGlyphRasterCallbackDenial> {
            Err(worth_ui_host_contract::UiGlyphRasterCallbackDenial::Rejected)
        }
    }

    let mut rasterizer = NoMissRasterizer;
    perform(
        state,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::from_pin_release(
            request,
        ),
        &[],
        pins,
        &mut rasterizer,
    )
}

#[cfg(test)]
fn perform_with_upload_port(
    state: &mut UiNativeHostState,
    presentation_basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
    demands: &[UiGlyphRasterDemandBatchView<'_>],
    pins: UiGlyphRasterPinTransitionView<'_>,
    rasterizer: &mut dyn UiGlyphRasterMissRasterizer,
    upload_port: &mut dyn UiNativeTextAtlasUploadPort,
) -> UiGlyphRasterTransactionOutcome {
    text_atlas_transaction::perform(text_atlas_transaction::TextAtlasExecution {
        state,
        presentation_basis,
        demands,
        pins,
        rasterizer,
        upload_port,
    })
}

#[cfg(test)]
#[path = "text_atlas_tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::seed_pending_atlas_for_event_loop;

#[path = "text_atlas_upload_sink.rs"]
mod text_atlas_upload_sink;

#[path = "text_atlas_upload.rs"]
mod text_atlas_upload;

#[path = "text_atlas_physical_ownership.rs"]
mod text_atlas_physical_ownership;

#[cfg(test)]
#[path = "text_atlas_dx12_upload_port.rs"]
mod text_atlas_dx12_upload_port;

#[path = "text_atlas_admission.rs"]
mod text_atlas_admission;

#[path = "text_atlas_rasterization.rs"]
mod text_atlas_rasterization;

#[path = "text_atlas_settlement.rs"]
mod text_atlas_settlement;

#[path = "text_atlas_transaction.rs"]
mod text_atlas_transaction;

#[cfg(feature = "certification-support")]
#[path = "text_foreground_atlas_model.rs"]
mod text_foreground_atlas_model;
#[cfg(feature = "certification-support")]
pub use text_foreground_atlas_model::{
    UiNativeTextForegroundAtlasModel, UiNativeTextForegroundCoverageCertification,
    UiNativeTextForegroundFinalizationDenial, UiNativeTextForegroundJoinCost,
    UiNativeTextReplayOperation, UiNativeTextRetentionCertificationDenial,
};
