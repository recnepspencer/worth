use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::interaction::UiHostInteractionIngressOutcome;
use worth_ui::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationMountedBasis, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
    UiHostObservationTimeBasis, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfacePosition,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UiHostScrollDeltaSource,
    UiHostScrollDeltaTargetAffinity,
};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedHitTestMechanic, UiPresentationDeadline,
    UiSurfaceBindingGeneration,
};
use worth_ui_test_support::{
    WorthUiMountedIdentityCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::super::filesystem_mounted_world::{
    establish_allocation, launch_clipped_world, launch_world, prepare_frame,
};
use super::super::mounted_application_lifecycle::published_mounted_world::presented_epoch;

pub(super) struct InteractionWorld {
    pub(super) session: WorthUiActiveApplicationSession,
    pub(super) binding: UiSurfaceBindingGeneration,
    pub(super) presentation: UiHostObservationPresentationBasis,
    pub(super) hit_rows: Box<[UiMountedHitTestMechanic]>,
    next_sequence: u64,
}

impl InteractionWorld {
    pub(super) fn canonical() -> Self {
        Self::launch(launch_world())
    }

    pub(super) fn clipped() -> Self {
        Self::launch(launch_clipped_world())
    }

    pub(super) fn from_session(session: WorthUiActiveApplicationSession) -> Self {
        Self::launch(session)
    }

    fn launch(mut session: WorthUiActiveApplicationSession) -> Self {
        establish_allocation(&mut session, 3);
        let (presentation, binding, hit_rows) = publish(&mut session);
        Self {
            session,
            binding,
            presentation,
            hit_rows,
            next_sequence: 1,
        }
    }

    pub(super) fn publish_successor(&mut self) {
        let (presentation, binding, hit_rows) = publish(&mut self.session);
        assert_eq!(binding, self.binding);
        self.presentation = presentation;
        self.hit_rows = hit_rows;
    }

    pub(super) fn button(
        &mut self,
        pointer: u64,
        capture_epoch: u64,
        transition: UiHostPointerButtonTransition,
        point: [i64; 2],
    ) -> UiHostInteractionIngressOutcome {
        self.admit(
            self.presentation,
            UiHostObservationLoss::Complete,
            vec![UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(capture_epoch),
                button: UiHostPointerButton::Primary,
                transition,
                position: position(point),
            }],
        )
    }

    pub(super) fn button_with_time_basis(
        &mut self,
        pointer: u64,
        capture_epoch: u64,
        transition: UiHostPointerButtonTransition,
        point: [i64; 2],
        time_basis: UiHostObservationTimeBasis,
    ) -> UiHostInteractionIngressOutcome {
        let sequence = UiHostObservationSequence::new(self.next_sequence);
        self.next_sequence += 1;
        let report = UiHostObservationReport::new(
            sequence,
            time_basis,
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(capture_epoch),
                button: UiHostPointerButton::Primary,
                transition,
                position: position(point),
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("button reports carry an explicit pointer device kind");
        self.admit_range(
            self.presentation,
            (sequence, sequence),
            UiHostObservationLoss::Complete,
            vec![report],
        )
    }

    pub(super) fn motion(
        &mut self,
        pointer: u64,
        capture_epoch: u64,
        point: [i64; 2],
    ) -> UiHostInteractionIngressOutcome {
        self.admit(
            self.presentation,
            UiHostObservationLoss::Complete,
            vec![UiHostObservationPayload::PointerMotion {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(capture_epoch),
                pressed_buttons:
                    worth_ui::facade::observation_report::UiHostPressedPointerButtons::from_buttons(
                        [UiHostPointerButton::Primary],
                    ),
                position: position(point),
            }],
        )
    }

    pub(super) fn focus_loss(&mut self) -> UiHostInteractionIngressOutcome {
        self.admit(
            self.presentation,
            UiHostObservationLoss::Complete,
            vec![self.window_focus(false)],
        )
    }

    pub(super) fn scroll(
        &mut self,
        point: [i64; 2],
        block_subpixels: i64,
    ) -> UiHostInteractionIngressOutcome {
        self.admit(
            self.presentation,
            UiHostObservationLoss::Complete,
            vec![UiHostObservationPayload::ScrollDelta {
                source: UiHostScrollDeltaSource::PointerWheel,
                phase: UiHostScrollDeltaPhase::Updated,
                precision: UiHostScrollDeltaPrecision::Pixel,
                target: UiHostScrollDeltaTargetAffinity::exact_coordinate(
                    self.presentation,
                    position(point),
                ),
                x_subpixels: 0,
                y_subpixels: block_subpixels,
            }],
        )
    }

    pub(super) fn window_focus(&self, focused: bool) -> UiHostObservationPayload {
        UiHostObservationPayload::WindowFocus {
            surface: self.presentation.host_surface(),
            focused,
        }
    }

    pub(super) fn payload_at(
        &mut self,
        sequence: u64,
        tick: u64,
        payload: UiHostObservationPayload,
    ) -> UiHostInteractionIngressOutcome {
        let sequence = UiHostObservationSequence::new(sequence);
        let report = UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(tick),
            payload,
        );
        self.admit_range(
            self.presentation,
            (sequence, sequence),
            UiHostObservationLoss::Complete,
            vec![report],
        )
    }

    pub(super) fn pointer_button_overflow(&mut self) -> UiHostInteractionIngressOutcome {
        let sequence = UiHostObservationSequence::new(self.next_sequence);
        self.next_sequence += 1;
        let loss = UiHostObservationLoss::Overflow {
            family: worth_ui::facade::observation_report::UiHostObservationFamily::PointerButton,
            affected: UiHostObservationSequenceRange::new(sequence, sequence),
        };
        self.admit_range(self.presentation, (sequence, sequence), loss, Vec::new())
    }

    pub(super) fn button_at_presentation(
        &mut self,
        presentation: UiHostObservationPresentationBasis,
        pointer: u64,
        transition: UiHostPointerButtonTransition,
        point: [i64; 2],
    ) -> UiHostInteractionIngressOutcome {
        self.admit(
            presentation,
            UiHostObservationLoss::Complete,
            vec![UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                button: UiHostPointerButton::Primary,
                transition,
                position: position(point),
            }],
        )
    }

    pub(super) fn button_with_mounted_basis(
        &mut self,
        mounted: UiHostObservationMountedBasis,
        pointer: u64,
        transition: UiHostPointerButtonTransition,
        point: [i64; 2],
    ) -> UiHostInteractionIngressOutcome {
        let sequence = UiHostObservationSequence::new(self.next_sequence);
        self.next_sequence += 1;
        let report = UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(pointer),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                button: UiHostPointerButton::Primary,
                transition,
                position: position(point),
            },
        )
        .with_mounted_basis(mounted)
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("mounted button reports carry an explicit pointer device kind");
        self.admit_range(
            self.presentation,
            (sequence, sequence),
            UiHostObservationLoss::Complete,
            vec![report],
        )
    }

    fn admit(
        &mut self,
        presentation: UiHostObservationPresentationBasis,
        loss: UiHostObservationLoss,
        payloads: Vec<UiHostObservationPayload>,
    ) -> UiHostInteractionIngressOutcome {
        let first = UiHostObservationSequence::new(self.next_sequence);
        let reports = payloads
            .into_iter()
            .map(|payload| {
                let sequence = UiHostObservationSequence::new(self.next_sequence);
                self.next_sequence += 1;
                let is_pointer = matches!(
                    &payload,
                    UiHostObservationPayload::PointerMotion { .. }
                        | UiHostObservationPayload::PointerButton { .. }
                );
                let report = UiHostObservationReport::new(
                    sequence,
                    UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
                    payload,
                );
                if is_pointer {
                    report
                        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
                        .expect("pointer reports carry an explicit pointer device kind")
                } else {
                    report
                }
            })
            .collect::<Vec<_>>();
        let last = reports
            .last()
            .map_or(first, UiHostObservationReport::sequence);
        self.admit_range(presentation, (first, last), loss, reports)
    }

    fn admit_range(
        &mut self,
        presentation: UiHostObservationPresentationBasis,
        sequences: (UiHostObservationSequence, UiHostObservationSequence),
        loss: UiHostObservationLoss,
        reports: Vec<UiHostObservationReport>,
    ) -> UiHostInteractionIngressOutcome {
        let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol: protocol(),
            host_session: self.session.host_session_identity().as_u64(),
            presentation,
            sequences: UiHostObservationSequenceRange::new(sequences.0, sequences.1),
            loss,
            reports,
        })
        .expect("gesture world emits a structurally valid raw batch");
        self.session.admit_host_interaction_batch(batch)
    }
}

fn publish(
    session: &mut WorthUiActiveApplicationSession,
) -> (
    UiHostObservationPresentationBasis,
    UiSurfaceBindingGeneration,
    Box<[UiMountedHitTestMechanic]>,
) {
    let prepared = prepare_frame(session).expect("gesture world completes mounted projection");
    let hit_rows = prepared.surfaces()[0]
        .projection()
        .hit_tests()
        .rows()
        .to_vec()
        .into_boxed_slice();
    let publication = match session.present_prepared_mounted_frame(
        prepared,
        UiPresentationDeadline::at_tick(1_000),
        0,
    ) {
        UiMountedFrameOutcome::Published(publication) => publication,
        UiMountedFrameOutcome::Unchanged(_) => panic!("gesture world returned unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("gesture world returned reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("gesture world was rejected before effects")
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("gesture world remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("gesture world presentation became indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => panic!("gesture world retention denied"),
        UiMountedFrameOutcome::AdmissionDenied(_) => panic!("gesture world admission denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => panic!("gesture world completion denied"),
        UiMountedFrameOutcome::Superseded(_) => panic!("gesture world was superseded"),
    };
    let binding = publication.bindings()[0];
    let epoch = presented_epoch(session, publication.frame(), binding);
    let presentation = UiHostObservationPresentationBasis::new(
        session.inspect_mounted_identity().surface_bindings()[0].host_surface_identity(),
        publication.frame(),
        binding,
        epoch,
    );
    (presentation, binding, hit_rows)
}

fn position(point: [i64; 2]) -> UiHostSurfacePosition {
    UiHostSurfacePosition::viewport_logical(
        point[0] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        point[1] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    )
}

fn protocol() -> worth_ui::facade::observation_report::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => {
            panic!("current host observation protocol must negotiate")
        }
    }
}
