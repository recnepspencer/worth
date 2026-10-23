//! Complete role-specific source linkage in the existing causal fact row.
use super::*;

pub(super) fn locators(
    layout: &WorthQueryAftermathCausalityLayout,
) -> [&worth_foundational::facade::AspectFieldLocator; 4] {
    [
        &layout.source_world_owner_locator,
        &layout.source_world_commit_locator,
        &layout.source_branch_locator,
        &layout.source_commit_locator,
    ]
}

pub(super) fn values(pending: &WorthQueryPendingAftermathCausality) -> [Option<AspectValue>; 4] {
    let Some(source) = pending.recorded_source() else {
        return [None, None, None, None];
    };
    [
        Some(AspectValue::UInt64(source.world().owner_identity().get())),
        Some(AspectValue::UInt64(source.world().ordinal())),
        Some(AspectValue::String(InternedString::from(
            source.relational().branch_id.0.clone(),
        ))),
        Some(AspectValue::UInt64(source.relational().commit_id.0)),
    ]
}

pub(super) fn append(
    layout: &WorthQueryAftermathCausalityLayout,
    pending: &WorthQueryPendingAftermathCausality,
    fields: &mut BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
) {
    for (locus, value) in locators(layout).into_iter().zip(values(pending)) {
        if let Some(value) = value {
            fields.insert(locus.clone(), value);
        }
    }
}
