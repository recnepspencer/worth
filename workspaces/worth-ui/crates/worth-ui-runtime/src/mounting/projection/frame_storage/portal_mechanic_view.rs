use worth_ui_host_contract::{
    UiMountedFilledRectMechanic, UiMountedFilledRectReference, UiMountedHitTestMechanic,
    UiMountedHitTestReference, UiMountedInstanceIdentity,
};

use super::super::UiMountedProjectionDenial;
use super::{
    portal_child_view::UiMountedPortalChildPresentation, UiMountedProjectionFrame,
    UiMountedProjectionSurface,
};

pub(super) type UiMountedFilledRectReferenceIndex =
    std::collections::BTreeMap<UiMountedInstanceIdentity, UiMountedFilledRectReference>;
pub(super) type UiMountedHitTestReferenceIndex =
    std::collections::BTreeMap<UiMountedInstanceIdentity, UiMountedHitTestReference>;

pub(super) struct UiMountedFilledRectViewRows {
    pub(super) rows: Vec<UiMountedFilledRectMechanic>,
    pub(super) references: UiMountedFilledRectReferenceIndex,
}

pub(super) struct UiMountedHitTestViewRows {
    pub(super) rows: Vec<UiMountedHitTestMechanic>,
    pub(super) references: UiMountedHitTestReferenceIndex,
}

impl UiMountedProjectionFrame {
    pub(super) fn filled_rect_view_rows(
        &self,
        surface: UiMountedProjectionSurface,
    ) -> Result<UiMountedFilledRectViewRows, UiMountedProjectionDenial> {
        let source_rows = self.mechanics.filled_rects_for(
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
                UiMountedPortalChildPresentation::Presented(portal) => rows.extend(
                    row.presented_within_portal(portal)
                        .map_err(UiMountedProjectionDenial::StaticPaintCompletion)?,
                ),
            }
        }
        let references = rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                u16::try_from(index)
                    .map(|index| {
                        (
                            row.mounted_instance(),
                            UiMountedFilledRectReference::from_runtime_mounting(index),
                        )
                    })
                    .map_err(|_| UiMountedProjectionDenial::StaticPaintCapacityExceeded)
            })
            .collect::<Result<UiMountedFilledRectReferenceIndex, _>>()?;
        Ok(UiMountedFilledRectViewRows { rows, references })
    }

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
                UiMountedPortalChildPresentation::Presented(portal) => rows.extend(
                    row.presented_within_portal(portal)
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

pub(super) fn portal_relative_allocation(
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> Result<worth_ui_host_contract::UiMountedAllocationProjection, UiMountedProjectionDenial> {
    let (bounds, basis, anchor) = match allocation {
        worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, basis } => {
            (bounds, basis, false)
        }
        worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
            bounds,
            basis,
        } => (bounds, basis, true),
        worth_ui_host_contract::UiMountedAllocationProjection::Omitted(reason) => {
            return Ok(worth_ui_host_contract::UiMountedAllocationProjection::Omitted(reason))
        }
    };
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: portal.bounds().x() + bounds.x(),
            y: portal.bounds().y() + bounds.y(),
            width: bounds.width(),
            height: bounds.height(),
            coordinate_space: portal.bounds().coordinate_space(),
        },
    )
    .map_err(|_| UiMountedProjectionDenial::NonFiniteGeometry)?;
    Ok(if anchor {
        worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
            bounds,
            basis,
        }
    } else {
        worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, basis }
    })
}
