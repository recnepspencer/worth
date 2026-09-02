use worth_ui_host_contract::{UiAppearanceDamageRegion, UiOverlayParticipantIdentity};

use super::fact::{
    UiMountedAppearanceDamageShape, UiMountedAppearanceFact, UiMountedAppearanceFacts,
};
use super::UiMountedAppearanceLoweringDenial;

pub(super) fn damage_for_change(
    predecessor: Option<&UiMountedAppearanceFacts>,
    successor: &UiMountedAppearanceFacts,
    changes: &[worth_ui_host_contract::UiMountedAppearanceMechanicChange],
    order_changed: bool,
) -> Result<Vec<UiAppearanceDamageRegion>, UiMountedAppearanceLoweringDenial> {
    let mut regions = Vec::new();
    for change in changes {
        let identity = match change {
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Insert(mechanic) => {
                mechanic.identity()
            }
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Replace {
                predecessor,
                ..
            }
            | worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(predecessor) => {
                predecessor.clone()
            }
        };
        if let Some(fact) = predecessor.and_then(|facts| facts.record(&identity)) {
            push_shape(&mut regions, fact)?;
        }
        if let Some(fact) = successor.record(&identity) {
            push_shape(&mut regions, fact)?;
        }
    }
    if order_changed {
        let first = first_order_difference(predecessor, successor);
        for participant in successor
            .frame()
            .overlay_order()
            .bottom_to_top()
            .iter()
            .skip(first)
        {
            if let Some(fact) = participant_fact(successor, participant) {
                push_shape(&mut regions, fact)?;
            }
        }
        if let Some(predecessor) = predecessor {
            for participant in predecessor
                .frame()
                .overlay_order()
                .bottom_to_top()
                .iter()
                .skip(first)
            {
                if let Some(fact) = participant_fact(predecessor, participant) {
                    push_shape(&mut regions, fact)?;
                }
            }
        }
    }
    normalize(regions)
}

fn push_shape(
    regions: &mut Vec<UiAppearanceDamageRegion>,
    fact: &UiMountedAppearanceFact,
) -> Result<(), UiMountedAppearanceLoweringDenial> {
    let shape = match fact.damage() {
        UiMountedAppearanceDamageShape::Visual { bounds, .. } => {
            UiAppearanceDamageRegion::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
                .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?
        }
        UiMountedAppearanceDamageShape::Backdrop { extent } => {
            UiAppearanceDamageRegion::new(extent.x(), extent.y(), extent.width(), extent.height())
                .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?
        }
        UiMountedAppearanceDamageShape::None => return Ok(()),
    };
    regions.push(shape);
    Ok(())
}

fn participant_fact<'a>(
    facts: &'a UiMountedAppearanceFacts,
    participant: &UiOverlayParticipantIdentity,
) -> Option<&'a UiMountedAppearanceFact> {
    match participant {
        UiOverlayParticipantIdentity::Portal(instance) => facts.records().iter().find(|fact| {
            matches!(
                fact.identity(),
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::PortalSurface(
                    candidate
                ) if candidate == instance
            )
        }),
        UiOverlayParticipantIdentity::Backdrop(identity) => facts.records().iter().find(|fact| {
            matches!(
                fact.identity(),
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Backdrop(candidate)
                    if candidate == identity
            )
        }),
    }
}

fn first_order_difference(
    predecessor: Option<&UiMountedAppearanceFacts>,
    successor: &UiMountedAppearanceFacts,
) -> usize {
    let Some(predecessor) = predecessor else {
        return 0;
    };
    predecessor
        .frame()
        .overlay_order()
        .bottom_to_top()
        .iter()
        .zip(successor.frame().overlay_order().bottom_to_top())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| {
            predecessor
                .frame()
                .overlay_order()
                .bottom_to_top()
                .len()
                .min(successor.frame().overlay_order().bottom_to_top().len())
        })
}

fn normalize(
    mut regions: Vec<UiAppearanceDamageRegion>,
) -> Result<Vec<UiAppearanceDamageRegion>, UiMountedAppearanceLoweringDenial> {
    regions.sort_by_key(|region| (region.x(), region.y(), region.width(), region.height()));
    let mut normalized = Vec::with_capacity(regions.len());
    for region in regions {
        let mut merged = region;
        let mut index = 0;
        while index < normalized.len() {
            if touches_or_overlaps(normalized[index], merged) {
                merged = union(normalized.remove(index), merged)?;
                index = 0;
            } else {
                index += 1;
            }
        }
        normalized.push(merged);
    }
    normalized.sort_by_key(|region| (region.x(), region.y(), region.width(), region.height()));
    Ok(normalized)
}

fn touches_or_overlaps(left: UiAppearanceDamageRegion, right: UiAppearanceDamageRegion) -> bool {
    let left_x = i64::from(left.x());
    let left_y = i64::from(left.y());
    let left_right = left_x + i64::from(left.width());
    let left_bottom = left_y + i64::from(left.height());
    let right_x = i64::from(right.x());
    let right_y = i64::from(right.y());
    let right_right = right_x + i64::from(right.width());
    let right_bottom = right_y + i64::from(right.height());
    let overlaps = left_x < right_right
        && right_x < left_right
        && left_y < right_bottom
        && right_y < left_bottom;
    let horizontal_edge = (left_bottom == right_y || right_bottom == left_y)
        && left_x.max(right_x) < left_right.min(right_right);
    let vertical_edge = (left_right == right_x || right_right == left_x)
        && left_y.max(right_y) < left_bottom.min(right_bottom);
    overlaps || horizontal_edge || vertical_edge
}

fn union(
    left: UiAppearanceDamageRegion,
    right: UiAppearanceDamageRegion,
) -> Result<UiAppearanceDamageRegion, UiMountedAppearanceLoweringDenial> {
    let x = i64::from(left.x()).min(i64::from(right.x()));
    let y = i64::from(left.y()).min(i64::from(right.y()));
    let right_edge = (i64::from(left.x()) + i64::from(left.width()))
        .max(i64::from(right.x()) + i64::from(right.width()));
    let bottom_edge = (i64::from(left.y()) + i64::from(left.height()))
        .max(i64::from(right.y()) + i64::from(right.height()));
    UiAppearanceDamageRegion::new(
        i32::try_from(x).map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?,
        i32::try_from(y).map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?,
        u32::try_from(right_edge - x)
            .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?,
        u32::try_from(bottom_edge - y)
            .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?,
    )
    .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)
}
