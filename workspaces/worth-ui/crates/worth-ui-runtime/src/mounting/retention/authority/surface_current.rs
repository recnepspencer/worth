use super::super::UiPresentedFrameBasisDenial;
use super::*;
use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiSemanticSurfaceIdentity};

impl UiMountedFrameRetentionAuthority {
    pub(in crate::mounting::retention) fn surface_evidence(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<&UiRetainedPresentedFrame> {
        match self.frame(*self.frames.surface_frames.get(&surface)?) {
            UiMountedRetainedFrameLookup::Found { evidence, .. } => Some(evidence),
            _ => None,
        }
    }

    pub(in crate::mounting::retention) fn surface_for_current_presentation(
        &self,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<UiSemanticSurfaceIdentity, UiPresentedFrameBasisDenial> {
        let evidence = match self.frame(presentation.frame()) {
            UiMountedRetainedFrameLookup::Found { evidence, .. } => evidence,
            UiMountedRetainedFrameLookup::Expired { .. } => {
                return Err(UiPresentedFrameBasisDenial::Expired)
            }
            UiMountedRetainedFrameLookup::Unknown { .. } => {
                return Err(UiPresentedFrameBasisDenial::Unknown)
            }
        };
        evidence.classify(presentation, None, None)?;
        evidence
            .current_presentations()
            .find_map(|(surface, current)| {
                (current == presentation
                    && self.frames.surface_frames.get(&surface) == Some(&presentation.frame()))
                .then_some(surface)
            })
            .ok_or(UiPresentedFrameBasisDenial::Expired)
    }

    pub(in crate::mounting::retention) fn evidence_rc(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Option<Rc<UiRetainedPresentedFrame>> {
        self.frames
            .current
            .as_ref()
            .filter(|current| current.frame() == frame)
            .cloned()
            .or_else(|| self.frames.predecessors.get(&frame).cloned())
    }

    pub(in crate::mounting::retention) fn replace_evidence(
        &mut self,
        evidence: Rc<UiRetainedPresentedFrame>,
    ) {
        if self
            .frames
            .current
            .as_ref()
            .is_some_and(|current| current.frame() == evidence.frame())
        {
            self.frames.current = Some(evidence);
        } else {
            assert!(
                self.frames.predecessors.get(&evidence.frame()).is_some(),
                "updated evidence remains retained"
            );
            self.frames.predecessors.insert(evidence.frame(), evidence);
        }
    }
}
