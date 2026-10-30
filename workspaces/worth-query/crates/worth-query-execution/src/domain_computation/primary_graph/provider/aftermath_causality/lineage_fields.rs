//! Recorded correction relationships, plus reproducible native index selectors.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
    WorthQueryRecordedCorrectionSource,
};

pub(super) fn selector(source: &WorthQueryCorrectionSourceCoordinates) -> AspectValue {
    // Branch length makes the scalar tuple unambiguous even for punctuation in a branch name.
    AspectValue::String(InternedString::from(format!(
        "{}",
        Selector {
            owner: source.world_owner(),
            commit: source.world_commit(),
            receipt: source.relational(),
        }
    )))
}

pub(super) fn selector_matches<Stop>(
    source: &WorthQueryRecordedCorrectionSource,
    value: Option<&AspectValue>,
    mut admit: impl FnMut(u64) -> Result<(), Stop>,
) -> Result<bool, Stop> {
    use std::fmt::Write;
    let Some(AspectValue::String(InternedString::Raw(value))) = value else {
        return Ok(false);
    };
    // Four u64 decimal conversions have bounded stack output. Pay both actual
    // branch/string traversals before the shared grammar writes comparisons.
    admit(value.len() as u64)?;
    admit(source.relational().branch_id.0.len() as u64)?;
    admit(4 * 20 + 4)?;
    let mut matching = SelectorMatch {
        expected: value,
        offset: 0,
    };
    let result = write!(
        &mut matching,
        "{}",
        Selector {
            owner: source.world().owner_identity().get(),
            commit: source.world().ordinal(),
            receipt: source.relational(),
        }
    );
    Ok(result.is_ok() && matching.offset == value.len())
}

struct Selector<'a> {
    owner: u64,
    commit: u64,
    receipt: &'a worth_relational::facade::history::RelationalCommitReceipt,
}
impl std::fmt::Display for Selector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}:{}",
            self.owner,
            self.commit,
            self.receipt.branch_id.0.len(),
            self.receipt.branch_id.0,
            self.receipt.commit_id.0
        )
    }
}
struct SelectorMatch<'a> {
    expected: &'a str,
    offset: usize,
}
impl std::fmt::Write for SelectorMatch<'_> {
    fn write_str(&mut self, part: &str) -> std::fmt::Result {
        let remaining = self.expected.get(self.offset..).ok_or(std::fmt::Error)?;
        if !remaining.starts_with(part) {
            return Err(std::fmt::Error);
        }
        self.offset += part.len();
        Ok(())
    }
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
        Some(lineage.root()),
        lineage.selected_alternative(),
        lineage.active_reversal(),
        lineage.prior_transition(),
    ] {
        fields.extend(coordinates(source));
    }
    let reverse = pending.role() == WorthQueryAftermathCausalRole::RecordedReverse;
    fields.push(reverse.then(|| selector(&lineage.root().coordinates())));
    fields.push(reverse.then(|| selector(&pending.recorded_source().coordinates())));
    fields.push(
        lineage
            .active_reversal()
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
