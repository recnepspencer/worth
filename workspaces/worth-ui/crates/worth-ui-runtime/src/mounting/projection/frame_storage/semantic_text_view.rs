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
            match self.portal_child_presentation(
                row.mounted_instance(),
                surface.surface,
                surface.binding,
            )? {
                super::portal_child_view::UiMountedPortalChildPresentation::Ordinary => {
                    rows.push(row)
                }
                super::portal_child_view::UiMountedPortalChildPresentation::Suppressed => {}
                super::portal_child_view::UiMountedPortalChildPresentation::Presented(portal) => {
                    rows.extend(
                        row.presented_within_portal(portal)
                            .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?,
                    )
                }
            }
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
