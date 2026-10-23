//! Recorded correction relationships, plus reproducible native index selectors.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
    WorthQueryRecordedCorrectionSource,
};

pub(super) fn selector(source: &WorthQueryCorrectionSourceCoordinates) -> AspectValue {
    // Branch length makes the scalar tuple unambiguous even for punctuation in a branch name.
    AspectValue::String(InternedString::from(format!(
        "{}:{}:{}:{}:{}",
        source.world_owner(),
        source.world_commit(),
        source.relational().branch_id.0.len(),
        source.relational().branch_id.0,
        source.relational().commit_id.0
    )))
}
pub(super) fn consumption_selector(
    reversal: &worth_relational::facade::history::RelationalCommitReceipt,
) -> AspectValue {
    // The exact native owner and basis already scope this receipt. Full World linkage remains in the canonical row.
    AspectValue::String(InternedString::from(format!(
        "{}:{}:{}",
        reversal.branch_id.0.len(),
        reversal.branch_id.0,
        reversal.commit_id.0
    )))
}
pub(super) fn locators(
    layout: &WorthQueryAftermathCausalityLayout,
) -> Vec<&worth_foundational::facade::AspectFieldLocator> {
    let lineage = &layout.lineage;
    lineage
        .root
        .iter()
        .chain(&lineage.selected)
        .chain(&lineage.active)
        .chain(&lineage.prior)
        .chain([
            &lineage.reversed_root,
            &lineage.reversed_source,
            &lineage.consumed_reversal,
        ])
        .collect()
}
pub(super) fn values(pending: &WorthQueryPendingAftermathCausality) -> Vec<Option<AspectValue>> {
    let lineage = pending.correction_lineage();
    let mut fields = Vec::with_capacity(19);
    for source in [
        lineage.map(|l| l.root()),
        lineage.and_then(|l| l.selected_alternative()),
        lineage.and_then(|l| l.active_reversal()),
        lineage.and_then(|l| l.prior_transition()),
    ] {
        fields.extend(coordinates(source));
    }
    let reverse = pending.role() == WorthQueryAftermathCausalRole::RecordedReverse;
    fields.push(
        lineage
            .filter(|_| reverse)
            .map(|l| selector(&l.root().coordinates())),
    );
    fields.push(
        pending
            .recorded_source()
            .filter(|_| reverse)
            .map(|s| selector(&s.coordinates())),
    );
    fields.push(
        lineage
            .and_then(|l| l.active_reversal())
            .map(|s| consumption_selector(s.relational())),
    );
    fields
}
fn coordinates(source: Option<&WorthQueryRecordedCorrectionSource>) -> [Option<AspectValue>; 4] {
    source.map_or([None, None, None, None], |source| {
        [
            Some(AspectValue::UInt64(source.world().owner_identity().get())),
            Some(AspectValue::UInt64(source.world().ordinal())),
            Some(AspectValue::String(InternedString::from(
                source.relational().branch_id.0.clone(),
            ))),
            Some(AspectValue::UInt64(source.relational().commit_id.0)),
        ]
    })
}
pub(super) fn append(
    layout: &WorthQueryAftermathCausalityLayout,
    pending: &WorthQueryPendingAftermathCausality,
    fields: &mut BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
) {
    for (locator, value) in locators(layout).into_iter().zip(values(pending)) {
        if let Some(value) = value {
            fields.insert(locator.clone(), value);
        }
    }
}
