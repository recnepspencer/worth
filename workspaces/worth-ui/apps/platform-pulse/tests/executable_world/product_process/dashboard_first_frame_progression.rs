//! The dashboard entry: the same causal first publication as every launch,
//! adjudicated by the declared geometry the dashboard paints at rest instead
//! of the source-signal colour oracle.
use std::time::Instant;

use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT;

use crate::adjudication::{
    adjudicate_vertical_thumb, physical_px, ExecutableFirstFrameFailure, VerticalThumbEvidence,
};
use crate::external_observation::{
    NativeClientPixelCapture, ProcessBoundNativeClientAreaObservation,
};
use crate::failure_teardown::{
    teardown_native_bound_world, teardown_unbound_world, PulseExecutableWorldFailureReport,
    UnboundFailureWorldResources,
};

use super::first_frame_progression::{adjudicate_bound_first_frame, bind_first_frame_world};
use super::{
    AwaitingFirstFrame, DashboardAtRest, NativeBoundExecutableWorld, Published,
    PulseExecutableWorld,
};

impl PulseExecutableWorld<AwaitingFirstFrame> {
    /// Wait for the dashboard's first frame: causal publication, one process,
    /// the declared client extent at the observed DPI, and the Recent activity
    /// thumb painted at rest where the declared geometry puts it.
    pub(crate) fn await_dashboard_at_rest(
        self,
        deadline: Instant,
    ) -> Result<PulseExecutableWorld<Published<DashboardAtRest>>, PulseExecutableWorldFailureReport>
    {
        let AwaitingFirstFrame {
            installation,
            mut process,
            mut lifecycle,
            launch_started,
        } = self.state;
        let bound =
            match bind_first_frame_world(&mut process, &mut lifecycle, launch_started, deadline) {
                Ok(bound) => bound,
                Err(primary) => {
                    return Err(teardown_unbound_world(
                        primary,
                        UnboundFailureWorldResources::new(installation, process, lifecycle),
                    ))
                }
            };
        let evidence = match adjudicate_bound_first_frame(
            &mut process,
            &bound,
            deadline,
            adjudicate_dashboard_at_rest,
        ) {
            Ok(evidence) => evidence,
            Err(primary) => {
                return Err(teardown_native_bound_world(
                    primary,
                    UnboundFailureWorldResources::new(installation, process, lifecycle)
                        .bind_native(bound.platform, bound.native_client),
                ))
            }
        };
        Ok(PulseExecutableWorld {
            state: Published {
                world: NativeBoundExecutableWorld {
                    installation,
                    process,
                    lifecycle,
                    journey_started: launch_started,
                    platform: bound.platform,
                    native_client: bound.native_client,
                },
                stage: DashboardAtRest { evidence },
            },
        })
    }
}

/// The dashboard pixel oracle: the client area is the product's declared
/// extent at the observed DPI, and the Recent activity thumb rests at offset 0.
fn adjudicate_dashboard_at_rest(
    pixels: &NativeClientPixelCapture,
    client_area: ProcessBoundNativeClientAreaObservation,
) -> Result<VerticalThumbEvidence, ExecutableFirstFrameFailure> {
    let dpi = client_area.dpi();
    let expected =
        PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT.map(|points| physical_px(f64::from(points), dpi));
    let bounds = client_area.bounds();
    let observed = [i64::from(bounds.width()), i64::from(bounds.height())];
    if observed != expected {
        return Err(ExecutableFirstFrameFailure::DeclaredClientExtent { expected, observed });
    }
    adjudicate_vertical_thumb(pixels, dpi, 0.0)
        .map_err(ExecutableFirstFrameFailure::RestingScrollChrome)
}
