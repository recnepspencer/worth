use worth_ui_host_contract::{
    UiMountedHitTestMechanic, UiMountedHitTestReference, UiMountedInstanceIdentity,
};

use super::super::UiMountedProjectionDenial;
use super::{
    portal_child_view::UiMountedPortalChildPresentation, UiMountedProjectionFrame,
    UiMountedProjectionSurface,
};

pub(super) type UiMountedHitTestReferenceIndex =
    std::collections::BTreeMap<UiMountedInstanceIdentity, UiMountedHitTestReference>;

pub(super) struct UiMountedHitTestViewRows {
    pub(super) rows: Vec<UiMountedHitTestMechanic>,
    pub(super) references: UiMountedHitTestReferenceIndex,
}

impl UiMountedProjectionFrame {
    pub(super) fn hit_test_view_rows(
        &self,
        surface: UiMountedProjectionSurface,
    ) -> Result<UiMountedHitTestViewRows, UiMountedProjectionDenial> {
        let source_rows = self.mechanics.hit_tests_for(
            &self.semantic,
            surface.surface,
            surface.binding,
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
                UiMountedPortalChildPresentation::Ordinary => rows.push(row),
                UiMountedPortalChildPresentation::Suppressed => {}
                UiMountedPortalChildPresentation::Presented(portal, source_anchor) => rows.extend(
                    row.presented_within_portal(portal, source_anchor)
                        .map_err(UiMountedProjectionDenial::HitTestCompletion)?,
                ),
            }
        }
        let references = rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                u32::try_from(index)
                    .map(|index| {
                        (
                            row.mounted_instance(),
                            UiMountedHitTestReference::from_runtime_mounting(index),
                        )
                    })
                    .map_err(|_| UiMountedProjectionDenial::HitTestCapacityExceeded)
            })
            .collect::<Result<UiMountedHitTestReferenceIndex, _>>()?;
        Ok(UiMountedHitTestViewRows { rows, references })
    }
}
