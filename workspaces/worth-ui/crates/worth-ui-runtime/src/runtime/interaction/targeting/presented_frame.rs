use super::presented_target::{
    seal_target, UiPresentedInteractionTarget, UiPresentedInteractionTargetView,
    UiPresentedTargetFrameRelation,
};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostSurfaceCoordinateSpace, UiHostSurfaceCoordinateUnit,
    UiHostSurfacePosition, UiHostSurfacePositionBasis, UiMountedCoordinateSpace,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiInteractionTargetingDenial {
    ExpiredPresentation,
    UnknownPresentation,
    BindingNotPresented,
    PresentationEpochMismatch,
    PresentationTruthUnavailable,
    UnsupportedPositionBasis(UiHostSurfacePositionBasis),
    IncompatibleHitTestCoordinateSpace { row: UiMountedCoordinateSpace },
    NoTarget { hit_test_rows_considered: usize },
    AmbiguousHitTestOrder { rank: u32 },
    GraphTargetNotPresented,
    SurfaceNoLongerBound,
    BindingNoLongerCurrent,
    MountedInstanceNoLongerCurrent,
    MountedSurfaceAffinityChanged,
    HitTestNodeBudgetExceeded,
    HitTestCandidateBudgetExceeded,
    InvalidHitTestPoint,
}

pub(crate) fn resolve_presented_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: UiHostObservationPresentationBasis,
    position: UiHostSurfacePosition,
) -> Result<UiPresentedInteractionTarget, UiInteractionTargetingDenial> {
    require_viewport_logical(position.basis())?;
    let point = canonical_point(position);
    let basis = mounted
        .interaction_hit_test_candidates(presentation, point.map(f64::from))
        .map_err(|denial| match denial {
            crate::mounting::UiPresentedPointLookupDenial::Presentation(denial) => {
                map_presentation_denial(denial)
            }
            crate::mounting::UiPresentedPointLookupDenial::Query(denial) => {
                map_hit_query_denial(denial)
            }
        })?;
    debug_assert_eq!(basis.presentation(), presentation);
    let relation = map_relation(basis.relation());
    let rows = basis.rows();
    let mut selected: Option<crate::mounting::UiPresentedHitTestRow> = None;
    for row in rows {
        if row.bounds().coordinate_space() != UiMountedCoordinateSpace::Viewport {
            return Err(
                UiInteractionTargetingDenial::IncompatibleHitTestCoordinateSpace {
                    row: row.bounds().coordinate_space(),
                },
            );
        }
        if !contains(row.bounds(), point) || !contains(row.clip_bounds(), point) {
            continue;
        }
        if let Some(current) = selected {
            if current.order() == row.order() {
                return Err(UiInteractionTargetingDenial::AmbiguousHitTestOrder {
                    rank: row.order().rank(),
                });
            }
            if current.order() < row.order() {
                continue;
            }
        }
        selected = Some(*row);
    }
    let row = selected.ok_or(UiInteractionTargetingDenial::NoTarget {
        hit_test_rows_considered: rows.len(),
    })?;
    let current = mounted
        .admit_current_hit_target(row.mounted())
        .map_err(map_current_affinity_denial)?;
    Ok(seal_target(
        presentation,
        relation,
        current,
        row,
        rows.len(),
    ))
}

pub(crate) fn resolve_presented_focus_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: UiHostObservationPresentationBasis,
    target: worth_ui_host_contract::UiHostFocusPlacementTarget,
) -> Result<Option<UiPresentedInteractionTargetView>, UiInteractionTargetingDenial> {
    let basis = mounted
        .semantic_focus_placement_basis(presentation)
        .map_err(map_presentation_denial)?;
    let Some(row) = basis.rows().iter().copied().find(|row| {
        row.mounted_instance() == target.mounted_instance()
            && row.node_receipt() == target.node_receipt()
    }) else {
        return Ok(None);
    };
    let current = mounted
        .admit_current_hit_target(row.mounted())
        .map_err(map_current_affinity_denial)?;
    Ok(Some(
        seal_target(
            presentation,
            map_relation(basis.relation()),
            current,
            row,
            basis.rows().len(),
        )
        .view(),
    ))
}

pub(crate) fn resolve_presented_surface_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: UiHostObservationPresentationBasis,
) -> Result<UiPresentedInteractionTargetView, UiInteractionTargetingDenial> {
    let basis = mounted
        .interaction_hit_test_basis(presentation)
        .map_err(map_presentation_denial)?;
    let row = basis
        .rows()
        .first()
        .copied()
        .ok_or(UiInteractionTargetingDenial::GraphTargetNotPresented)?;
    let current = mounted
        .admit_current_hit_target(row.mounted())
        .map_err(map_current_affinity_denial)?;
    Ok(seal_target(
        presentation,
        map_relation(basis.relation()),
        current,
        row,
        basis.rows().len(),
    )
    .view())
}

pub(crate) fn require_current_presentation(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: UiHostObservationPresentationBasis,
) -> Result<(), UiInteractionTargetingDenial> {
    let relation = mounted
        .classify_interaction_presentation(presentation)
        .map_err(map_presentation_denial)?;
    if map_relation(relation) != UiPresentedTargetFrameRelation::Current {
        return Err(UiInteractionTargetingDenial::ExpiredPresentation);
    }
    Ok(())
}

fn require_viewport_logical(
    basis: UiHostSurfacePositionBasis,
) -> Result<(), UiInteractionTargetingDenial> {
    let supported = basis.coordinate_space() == UiHostSurfaceCoordinateSpace::Viewport
        && basis.coordinate_unit() == UiHostSurfaceCoordinateUnit::LogicalPoint;
    supported
        .then_some(())
        .ok_or(UiInteractionTargetingDenial::UnsupportedPositionBasis(
            basis,
        ))
}

fn canonical_point(position: UiHostSurfacePosition) -> [f32; 2] {
    let scale = UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    [
        (position.x_subpixels() as f64 / scale) as f32,
        (position.y_subpixels() as f64 / scale) as f32,
    ]
}

fn contains(bounds: worth_ui_host_contract::UiMountedCanonicalBox, point: [f32; 2]) -> bool {
    point[0] >= bounds.x()
        && point[0] < bounds.x() + bounds.width()
        && point[1] >= bounds.y()
        && point[1] < bounds.y() + bounds.height()
}

pub(crate) fn map_current_affinity_denial(
    denial: crate::mounting::UiCurrentHitTargetAffinityDenial,
) -> UiInteractionTargetingDenial {
    match denial {
        crate::mounting::UiCurrentHitTargetAffinityDenial::PresentationNotCurrent => {
            UiInteractionTargetingDenial::ExpiredPresentation
        }
        crate::mounting::UiCurrentHitTargetAffinityDenial::SurfaceNoLongerBound => {
            UiInteractionTargetingDenial::SurfaceNoLongerBound
        }
        crate::mounting::UiCurrentHitTargetAffinityDenial::BindingNoLongerCurrent => {
            UiInteractionTargetingDenial::BindingNoLongerCurrent
        }
        crate::mounting::UiCurrentHitTargetAffinityDenial::MountedInstanceNoLongerCurrent => {
            UiInteractionTargetingDenial::MountedInstanceNoLongerCurrent
        }
        crate::mounting::UiCurrentHitTargetAffinityDenial::MountedSurfaceAffinityChanged => {
            UiInteractionTargetingDenial::MountedSurfaceAffinityChanged
        }
    }
}

fn map_relation(
    relation: crate::mounting::UiPresentedFrameBasisRelation,
) -> UiPresentedTargetFrameRelation {
    match relation {
        crate::mounting::UiPresentedFrameBasisRelation::Current => {
            UiPresentedTargetFrameRelation::Current
        }
        crate::mounting::UiPresentedFrameBasisRelation::Retained => {
            UiPresentedTargetFrameRelation::Retained
        }
    }
}

pub(super) fn map_presentation_denial(
    denial: crate::mounting::UiPresentedFrameBasisDenial,
) -> UiInteractionTargetingDenial {
    match denial {
        crate::mounting::UiPresentedFrameBasisDenial::Expired => {
            UiInteractionTargetingDenial::ExpiredPresentation
        }
        crate::mounting::UiPresentedFrameBasisDenial::Unknown => {
            UiInteractionTargetingDenial::UnknownPresentation
        }
        crate::mounting::UiPresentedFrameBasisDenial::BindingNotPresented => {
            UiInteractionTargetingDenial::BindingNotPresented
        }
        crate::mounting::UiPresentedFrameBasisDenial::PresentationEpochMismatch => {
            UiInteractionTargetingDenial::PresentationEpochMismatch
        }
        crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable => {
            UiInteractionTargetingDenial::PresentationTruthUnavailable
        }
        crate::mounting::UiPresentedFrameBasisDenial::InstanceNotPresented
        | crate::mounting::UiPresentedFrameBasisDenial::NodeReceiptMismatch => {
            unreachable!("target lookup classifies only frame presentation evidence")
        }
    }
}

pub(crate) fn require_current_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    target: UiPresentedInteractionTargetView,
) -> Result<(), UiInteractionTargetingDenial> {
    admit_current_target(mounted, target).map(|_| ())
}

pub(crate) fn admit_current_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    target: UiPresentedInteractionTargetView,
) -> Result<crate::mounting::UiCurrentInteractionAffinity, UiInteractionTargetingDenial> {
    mounted
        .admit_current_interaction_affinity(crate::mounting::UiMountedInteractionAffinityInput {
            surface: target.surface(),
            binding: target.binding(),
            mounted_instance: target.mounted_instance(),
            node_receipt: target.node_receipt(),
        })
        .map_err(map_current_affinity_denial)
}

pub(crate) fn admit_current_target_incarnation(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    target: UiPresentedInteractionTargetView,
) -> Result<crate::mounting::UiCurrentInteractionAffinity, UiInteractionTargetingDenial> {
    mounted
        .admit_current_mounted_incarnation_affinity(
            crate::mounting::UiMountedIncarnationAffinityInput {
                surface: target.surface(),
                binding: target.binding(),
                mounted_instance: target.mounted_instance(),
            },
        )
        .map_err(map_current_affinity_denial)
}

pub(crate) fn map_hit_query_denial(
    denial: crate::mounting::UiPresentedHitQueryDenial,
) -> UiInteractionTargetingDenial {
    match denial {
        crate::mounting::UiPresentedHitQueryDenial::IncompatibleCoordinateSpace(row) => {
            UiInteractionTargetingDenial::IncompatibleHitTestCoordinateSpace { row }
        }
        crate::mounting::UiPresentedHitQueryDenial::InvalidPoint => {
            UiInteractionTargetingDenial::InvalidHitTestPoint
        }
        crate::mounting::UiPresentedHitQueryDenial::NodeBudget { .. } => {
            UiInteractionTargetingDenial::HitTestNodeBudgetExceeded
        }
        crate::mounting::UiPresentedHitQueryDenial::CandidateBudget { .. } => {
            UiInteractionTargetingDenial::HitTestCandidateBudgetExceeded
        }
    }
}
