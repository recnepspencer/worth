//! Canonical correction relationships and the selectors derived from them.
use super::*;
use worth_foundational::facade::StructFieldBuilder;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCorrectionLineageLayout {
    pub root: [AspectFieldLocator; 4],
    pub selected: [AspectFieldLocator; 4],
    pub active: [AspectFieldLocator; 4],
    pub prior: [AspectFieldLocator; 4],
    pub reversed_root: AspectFieldLocator,
    pub reversed_source: AspectFieldLocator,
    pub consumed_reversal: AspectFieldLocator,
    pub reversal_root_index: DerivedIndexId,
    pub reversed_source_index: DerivedIndexId,
    pub consumed_reversal_index: DerivedIndexId,
}

pub(super) fn lower(
    mut shape: StructFieldBuilder,
) -> Result<
    (StructFieldBuilder, WorthQueryCorrectionLineageLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    for prefix in [
        "lineage-root",
        "selected-alternative",
        "active-reversal",
        "prior-transition",
    ] {
        for suffix in ["world-owner", "world-commit", "branch", "commit"] {
            let value_type = if suffix == "branch" {
                ScalarAspectType::String
            } else {
                ScalarAspectType::UInt64
            };
            shape = shape.optional(format!("{prefix}-{suffix}"), value_type);
        }
    }
    for field in ["reversed-root", "reversed-source", "consumed-reversal"] {
        shape = shape.optional(field, ScalarAspectType::String);
    }
    Ok((
        shape,
        WorthQueryCorrectionLineageLayout {
            root: coordinates("lineage-root")?,
            selected: coordinates("selected-alternative")?,
            active: coordinates("active-reversal")?,
            prior: coordinates("prior-transition")?,
            reversed_root: planned_field_locator(ASPECT, "reversed-root")?,
            reversed_source: planned_field_locator(ASPECT, "reversed-source")?,
            consumed_reversal: planned_field_locator(ASPECT, "consumed-reversal")?,
            reversal_root_index: DerivedIndexId(0),
            reversed_source_index: DerivedIndexId(0),
            consumed_reversal_index: DerivedIndexId(0),
        },
    ))
}

fn coordinates(
    prefix: &str,
) -> Result<[AspectFieldLocator; 4], WorthQueryPrimaryGraphInstallationDenial> {
    Ok([
        planned_field_locator(ASPECT, &format!("{prefix}-world-owner"))?,
        planned_field_locator(ASPECT, &format!("{prefix}-world-commit"))?,
        planned_field_locator(ASPECT, &format!("{prefix}-branch"))?,
        planned_field_locator(ASPECT, &format!("{prefix}-commit"))?,
    ])
}
