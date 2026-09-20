//! Publication of prepared frames through the shared World: settled
//! publications, and presentations the scripted host holds open so a test can
//! act while the host still owns the outcome.

use super::session::World;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::mounting::{UiMountedFrameOutcome, UiPreparedMountedFrame};
use worth_ui_host_contract::*;

impl World {
    pub(super) fn publish(&mut self, frame: UiPreparedMountedFrame, now: u64, initial: bool) {
        for _ in frame.surfaces() {
            if initial {
                self.host.push_native_display_presented();
            } else {
                self.host.push_native_display_settled_without_effects();
            }
        }
        let outcome = self.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
        );
        match outcome {
            UiMountedFrameOutcome::Published(_) => {}
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("shared publication at {now}: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(denial) => {
                panic!("shared publication: {:?}", denial.rejections())
            }
            other => panic!(
                "shared publication at {now}: {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    pub(super) fn publish_in_flight(&mut self, frame: UiPreparedMountedFrame, now: u64) {
        let predecessor = self.session.current_mounted_publication().unwrap().frame();
        let appearance = self
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .cloned();
        let pending = self.begin_in_flight(frame, now);
        assert_eq!(
            self.session.current_mounted_publication().unwrap().frame(),
            predecessor,
            "the accepted predecessor stays authoritative while host work is pending"
        );
        assert_eq!(
            self.session
                .mounted
                .current_unpublished_appearance()
                .unwrap(),
            appearance.as_ref(),
            "pending text and overlay work cannot replace accepted appearance"
        );
        let completed = self.session.complete_mounted_presentation(pending, now + 1);
        match completed {
            UiMountedFrameOutcome::Published(_) => {}
            UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                panic!("shared in-flight completion: {:?}", frame.report())
            }
            other => panic!(
                "shared in-flight completion: {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    /// Present one surface's `frame` against a host that holds its completion
    /// open, and hand back the in-flight attempt so the caller can act while
    /// the host still owns the outcome.
    pub(super) fn begin_in_flight(
        &mut self,
        frame: UiPreparedMountedFrame,
        now: u64,
    ) -> crate::mounting::UiMountedPresentationInFlight {
        assert_eq!(
            frame.surfaces().len(),
            1,
            "shared in-flight proof is one surface"
        );
        self.host.push_in_flight(
            vec![ScriptedSurfaceCompletion::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(now + 1_000),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            )],
            UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
        );
        let outcome = self.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            now,
        );
        match outcome {
            UiMountedFrameOutcome::InFlight(pending) => pending,
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("shared in-flight admission: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(denial) => {
                panic!("shared in-flight rejection: {:?}", denial.rejections())
            }
            other => panic!(
                "shared authored publication must enter the legal in-flight posture: {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}
