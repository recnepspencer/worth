use super::TextWorld;
use crate::mounting::*;
use worth_ui_host_contract::*;

impl TextWorld {
    pub fn reconstruct_current_text_layouts(&mut self, index: usize) {
        let (basis, _, _) = self.accepted[index].as_ref().unwrap();
        assert_eq!(
            self.session.current_mounted_publication().unwrap().frame(),
            basis.frame(),
            "layout loss targets the surface whose projection is currently owned"
        );
        let lost = self
            .session
            .mounted
            .require_current_layout_reconstruction(basis.binding())
            .unwrap();
        let expected = self
            .occurrences
            .iter()
            .filter(|(_, surface, _)| *surface == index)
            .count()
            * 2;
        assert_eq!(
            lost, expected,
            "each occurrence loses its value and posture layouts"
        );
        assert_eq!(
            self.session.mounted.reconstruct_current_layouts().unwrap(),
            expected
        );
    }

    pub fn reconstruct_after_rejection(&mut self, index: usize, expected: &str, tick: u64) {
        let surface = self.surfaces[index];
        let predecessor = self.accepted[index].as_ref().unwrap().0;
        let predecessor_frame = self.session.current_mounted_publication().unwrap().frame();
        let replacement = self
            .session
            .rebind_host_surface(
                predecessor.binding(),
                UiHostSurfacePresentationMode::NativeDisplay,
                UiSurfaceBindingProfile::new(
                    1_000,
                    UiSurfaceBindingCoordinatePosture::LogicalPoints,
                    1,
                )
                .unwrap(),
            )
            .unwrap()
            .binding_generation();
        let replacements = [UiMountedSurfaceReconciliationBinding::new(
            predecessor.binding(),
            replacement,
        )];
        for (offset, accept) in [false, true].into_iter().enumerate() {
            let frame = self
                .session
                .prepare_mounted_reconstruction_frame_with_application_presentation(
                    UiMountedFrameRequest::exact_surfaces(vec![surface]),
                    &replacements,
                    |_| {},
                )
                .unwrap_or_else(|_| {
                    panic!("text reconstruction must prepare through the production handoff")
                });
            if accept {
                self.host.push_native_display_presented();
            } else {
                self.host.push_rejected();
            }
            let outcome = self
                .session
                .present_prepared_mounted_reconstruction_frame(
                    frame,
                    &replacements,
                    UiPresentationDeadline::at_tick(u64::MAX),
                    tick + offset as u64,
                )
                .unwrap();
            match outcome {
                UiMountedFrameOutcome::RejectedBeforeEffects(_) if !accept => {
                    assert_eq!(
                        self.session.current_mounted_publication().unwrap().frame(),
                        predecessor_frame
                    );
                }
                UiMountedFrameOutcome::Reconciled(receipt) if accept => {
                    let accepted = self
                        .session
                        .mounted
                        .current_presentation_for_surface(surface)
                        .unwrap();
                    assert_eq!(accepted.frame(), receipt.frame());
                    assert_eq!(accepted.binding(), replacement);
                    self.accepted[index] = Some((
                        accepted,
                        receipt.attempt(),
                        self.session
                            .mounted
                            .current_projection_rc_for_test()
                            .unwrap(),
                    ));
                }
                UiMountedFrameOutcome::AdmissionDenied(denial) => {
                    panic!("reconstruction admission: {:?}", denial.denial())
                }
                other => panic!(
                    "unexpected reconstruction: {:?}",
                    std::mem::discriminant(&other)
                ),
            }
            self.assert_text(index, expected);
            self.assert_pending(None);
        }
    }
}
