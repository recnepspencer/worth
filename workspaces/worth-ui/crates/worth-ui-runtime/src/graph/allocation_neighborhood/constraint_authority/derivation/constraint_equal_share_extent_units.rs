//! The measured extent an equal share divides, counted in whole units.

use crate::evidence::{UiConstraintAxisScope, UiMeasurementBasis, UiMeasurementValue};

/// Whether sharing the measured extent among `peer_count` peers leaves a
/// remainder: the extent is unmeasured, is not a whole count of units, or
/// does not divide evenly.
pub(super) fn fractional_remainder_required(
    measurement_basis: &UiMeasurementBasis,
    axis_scope: UiConstraintAxisScope,
    primary_axis: crate::evidence::UiLayoutOperatorPrimaryAxis,
    peer_count: usize,
) -> bool {
    if peer_count == 0 {
        return false;
    }
    measurement_basis
        .evidence_inputs()
        .iter()
        .filter_map(|input| input.as_host_measurement_result())
        .find_map(|result| extent_for(result.value(), axis_scope, primary_axis))
        .is_none_or(|extent| whole_units(extent).is_none_or(|units| units % peer_count != 0))
}

/// The extent `axis_scope` shares out of a measured viewport, or `None` when
/// the value measures none or the scope names no axis.
fn extent_for(
    value: &UiMeasurementValue,
    axis_scope: UiConstraintAxisScope,
    primary_axis: crate::evidence::UiLayoutOperatorPrimaryAxis,
) -> Option<f32> {
    let (width, height) = match value {
        UiMeasurementValue::ViewportExtent(value) => (value.width, value.height),
        UiMeasurementValue::ScrollContainerViewport(value) => (value.width, value.height),
        _ => return None,
    };
    match axis_scope {
        UiConstraintAxisScope::Primary => Some(primary_extent(width, height, primary_axis)),
        UiConstraintAxisScope::Cross => cross_extent(width, height, primary_axis),
        UiConstraintAxisScope::Both => Some(width.min(height)),
    }
}

fn primary_extent(
    width: f32,
    height: f32,
    primary_axis: crate::evidence::UiLayoutOperatorPrimaryAxis,
) -> f32 {
    match primary_axis {
        crate::evidence::UiLayoutOperatorPrimaryAxis::Horizontal => width,
        crate::evidence::UiLayoutOperatorPrimaryAxis::Vertical
        | crate::evidence::UiLayoutOperatorPrimaryAxis::TwoDimensional
        | crate::evidence::UiLayoutOperatorPrimaryAxis::Layered
        | crate::evidence::UiLayoutOperatorPrimaryAxis::None => height,
    }
}

fn cross_extent(
    width: f32,
    height: f32,
    primary_axis: crate::evidence::UiLayoutOperatorPrimaryAxis,
) -> Option<f32> {
    Some(match primary_axis {
        crate::evidence::UiLayoutOperatorPrimaryAxis::Horizontal => height,
        crate::evidence::UiLayoutOperatorPrimaryAxis::Vertical => width,
        crate::evidence::UiLayoutOperatorPrimaryAxis::TwoDimensional
        | crate::evidence::UiLayoutOperatorPrimaryAxis::Layered => width.min(height),
        crate::evidence::UiLayoutOperatorPrimaryAxis::None => return None,
    })
}

/// `extent` as a count of whole units. `None` for an extent with a fraction,
/// which no split into whole units divides, and for one that is negative, not
/// finite, or past what `u32` counts.
fn whole_units(extent: f32) -> Option<usize> {
    crate::whole_number::whole_u32(f64::from(extent)).and_then(|units| usize::try_from(units).ok())
}
