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
        let Some(text) = node.semantic_text.as_ref() else {
            return Ok(Box::new([]));
        };
        let mut adopted = text.formatting().appearance_foreground_spans();
        if adopted.is_empty() {
            return Ok(Box::new([]));
        }
        let candidates = self.raw_appearance_text_candidates(instance)?;
        if !candidates.iter().any(|candidate| {
            if text.requires_lifecycle_caption() {
                candidate.in_layout_space().slot()
                    == worth_ui_host_contract::UiSemanticTextSlot::Posture
            } else {
                candidate.in_layout_space().slot()
                    != worth_ui_host_contract::UiSemanticTextSlot::Posture
            }
        }) {
            return Err(UiMountedAppearanceOutputDenial::TextCandidate(
                super::super::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
            ));
        }
        let mut retained = std::collections::HashSet::new();
        for candidate in candidates
            .iter()
            .filter(|candidate| !candidate.in_layout_space().text().is_empty())
        {
            let formatting = match candidate.in_layout_space().slot() {
                worth_ui_host_contract::UiSemanticTextSlot::Value => {
                    text.formatting().scalar_value_row()
                }
                worth_ui_host_contract::UiSemanticTextSlot::CollectionValue { .. }
                | worth_ui_host_contract::UiSemanticTextSlot::Posture => {
                    text.formatting().default_row()
                }
            };
            for span in formatting.appearance_foreground_spans() {
                if !candidate
                    .in_layout_space()
                    .foregrounds()
                    .iter()
                    .any(|row| row.identity() == span)
                {
                    return Err(UiMountedAppearanceOutputDenial::TextCandidate(
                        super::super::UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable,
                    ));
                }
                retained.insert(span);
            }
        }
        adopted = adopted
            .into_vec()
            .into_iter()
            .filter(|span| retained.contains(span))
            .collect();
        let candidates = self.clip_appearance_text_candidates(candidates)?;
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
        self.clip_appearance_text_candidates(candidates)
    }

    fn raw_appearance_text_candidates(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Result<
        Vec<crate::mounting::UiLaidOut<UiMountedSemanticTextMechanic>>,
        UiMountedAppearanceOutputDenial,
    > {
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
        candidates: Vec<crate::mounting::UiLaidOut<UiMountedSemanticTextMechanic>>,
    ) -> Result<Vec<UiMountedSemanticTextMechanic>, UiMountedAppearanceOutputDenial> {
        candidates
            .into_iter()
            .map(|candidate| self.present_semantic_text_row(candidate))
            .collect::<Result<Vec<_>, _>>()
            .map(|rows| rows.into_iter().flatten().collect())
    }
}
