use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedSemanticTextMechanic};

use super::super::appearance::{UiMountedAppearanceTextGeometry, UiMountedAppearanceTextSpanInput};
use super::{UiMountedAppearanceOutputDenial, UiMountedProjectionFrame};

impl UiMountedProjectionFrame {
    /// Adoption belongs to the declaration; candidate-visible membership comes
    /// from current qualified text after completed ancestor/Portal clipping.
    /// A partially clipped candidate retains its original spans; this does not
    /// infer individual glyph visibility from text ranges.
    pub(super) fn appearance_visible_foreground_spans(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Result<Box<[UiMountedAppearanceTextSpanInput]>, UiMountedAppearanceOutputDenial> {
        let node = self
            .semantic
            .node(instance)
            .ok_or(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
        let adopted = node.semantic_text.as_ref().map_or_else(
            || Box::new([]) as Box<[_]>,
            |text| text.appearance_foreground_spans(),
        );
        if adopted.is_empty() {
            return Ok(Box::new([]));
        }
        let candidates = self.raw_appearance_text_candidates(instance)?;
        let retained: std::collections::HashSet<_> = candidates
            .iter()
            .flat_map(|candidate| candidate.foregrounds().iter().map(|span| span.identity()))
            .collect();
        if adopted.iter().any(|span| !retained.contains(span)) {
            return Err(UiMountedAppearanceOutputDenial::TextCandidate(
                super::super::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
            ));
        }
        let candidates = self.clip_appearance_text_candidates(instance, candidates)?;
        let mut visible: std::collections::HashMap<_, Vec<_>> =
            adopted.iter().map(|span| (*span, Vec::new())).collect();
        for candidate in &candidates {
            for span in candidate.foregrounds() {
                let Some(geometry) = visible.get_mut(&span.identity()) else {
                    continue;
                };
                geometry.push(UiMountedAppearanceTextGeometry::from_candidate(
                    candidate,
                    span.original_range(),
                ));
            }
        }
        Ok(adopted
            .into_vec()
            .into_iter()
            .filter_map(|span| {
                visible
                    .remove(&span)
                    .filter(|geometry| !geometry.is_empty())
                    .map(|geometry| {
                        UiMountedAppearanceTextSpanInput::from_candidates(span, geometry)
                    })
            })
            .collect())
    }

    pub(super) fn appearance_text_candidates(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedAppearanceOutputDenial> {
        let candidates = self.raw_appearance_text_candidates(instance)?;
        self.clip_appearance_text_candidates(instance, candidates)
    }

    fn raw_appearance_text_candidates(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedAppearanceOutputDenial> {
        self.mechanics
            .semantic_text_for_instance(
                instance,
                self.content_generation(),
                self.frame_identity(),
                &self.receipt_basis,
            )
            .map_err(UiMountedAppearanceOutputDenial::TextCandidate)
    }

    fn clip_appearance_text_candidates(
        &self,
        instance: UiMountedInstanceIdentity,
        candidates: Vec<UiMountedSemanticTextMechanic>,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedAppearanceOutputDenial> {
        let node = self
            .semantic
            .node(instance)
            .ok_or(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
        let geometry = node.completed_appearance_geometry();
        use crate::mounting::projection::appearance::UiMountedAppearanceClip as Clip;
        if let Clip::Unresolved(denial) = geometry.clip {
            return Err(UiMountedAppearanceOutputDenial::AncestorClip(denial));
        }
        if geometry.clip == Clip::Suppressed {
            return Ok(Vec::new());
        }
        candidates
            .into_iter()
            .map(|candidate| {
                let presented = match geometry.portal_presentation {
                    Some(portal) => candidate.presented_within_portal(portal),
                    None => Ok(Some(candidate)),
                };
                presented
                    .and_then(|candidate| match (candidate, geometry.clip) {
                        (Some(candidate), Clip::Ancestor(clip)) => {
                            candidate.clipped_to_appearance_ancestor(clip)
                        }
                        (candidate, _) => Ok(candidate),
                    })
                    .map_err(|denial| {
                        UiMountedAppearanceOutputDenial::TextCandidate(
                            super::super::UiMountedProjectionDenial::SemanticTextCompletion(denial),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|rows| rows.into_iter().flatten().collect())
    }
}
