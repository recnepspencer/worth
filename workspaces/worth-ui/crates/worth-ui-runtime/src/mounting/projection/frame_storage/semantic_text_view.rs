use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedSemanticTextMechanic, UiMountedSemanticTextReference,
};

use super::{UiMountedProjectionDenial, UiMountedProjectionFrame, UiMountedProjectionSurface};

pub(super) type UiMountedSemanticTextReferenceIndex =
    std::collections::BTreeMap<UiMountedInstanceIdentity, Vec<UiMountedSemanticTextReference>>;

pub(super) struct UiMountedSemanticTextViewRows {
    pub(super) rows: Vec<UiMountedSemanticTextMechanic>,
    pub(super) references: UiMountedSemanticTextReferenceIndex,
}

impl UiMountedProjectionFrame {
    pub(super) fn source_text_deltas_are_presented(
        &self,
        instances: &[UiMountedInstanceIdentity],
    ) -> bool {
        instances.iter().all(|instance| self.semantic.node(*instance).is_some_and(|node| {
            let geometry = node.completed_appearance_geometry();
            geometry.placement() == crate::mounting::UiMountedPlacement::InPlace
                && geometry.clip == crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped
        }))
    }

    /// Ordinary commands, glyph attribution and appearance consume this same
    /// mounted presentation geometry. Qualification remains in source space.
    pub(super) fn present_semantic_text_row(
        &self,
        candidate: crate::mounting::UiLaidOut<UiMountedSemanticTextMechanic>,
    ) -> Result<Option<UiMountedSemanticTextMechanic>, super::UiMountedAppearanceOutputDenial> {
        use super::UiMountedAppearanceOutputDenial as Denial;
        use crate::mounting::projection::appearance::UiMountedAppearanceClip as Clip;
        let node = self
            .semantic
            .node(candidate.in_layout_space().mounted_instance())
            .ok_or(Denial::CurrentProjectionUnavailable)?;
        let geometry = node.completed_appearance_geometry();
        match geometry.clip {
            Clip::Unresolved(denial) => return Err(Denial::AncestorClip(denial)),
            Clip::Suppressed => return Ok(None),
            _ => {}
        }
        geometry
            .placement()
            .present(candidate)
            .and_then(|candidate| {
                match (
                    candidate.map(crate::mounting::UiPresented::into_shown),
                    geometry.clip,
                ) {
                    (Some(candidate), Clip::Ancestor(clip)) => {
                        candidate.clipped_to_appearance_ancestor(clip)
                    }
                    (candidate, _) => Ok(candidate),
                }
            })
            .map_err(|denial| {
                Denial::TextCandidate(UiMountedProjectionDenial::SemanticTextCompletion(denial))
            })
    }

    pub(super) fn semantic_text_view_rows(
        &self,
        surface: UiMountedProjectionSurface,
    ) -> Result<UiMountedSemanticTextViewRows, UiMountedProjectionDenial> {
        let source_rows = self.mechanics.semantic_text_for(
            &self.semantic,
            surface.surface,
            surface.binding,
            self.content_generation,
            self.frame,
            &self.receipt_basis,
        )?;
        let mut rows = Vec::with_capacity(source_rows.len());
        for row in source_rows {
            rows.extend(
                self.present_semantic_text_row(row)
                    .map_err(|_| UiMountedProjectionDenial::AppearanceTextCandidatesUnavailable)?,
            );
        }

        let mut references = UiMountedSemanticTextReferenceIndex::new();
        for (index, row) in rows.iter().enumerate() {
            let reference = u16::try_from(index)
                .map(UiMountedSemanticTextReference::from_runtime_mounting)
                .map_err(|_| UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            references
                .entry(row.mounted_instance())
                .or_default()
                .push(reference);
        }
        Ok(UiMountedSemanticTextViewRows { rows, references })
    }
}
