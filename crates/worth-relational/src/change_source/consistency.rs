use std::collections::{BTreeMap, HashMap};

use worth_foundational::facade::AuthoritativeAspectChangeKind;

use super::consistency_expectations::{
    consume_expected, expected_changes, semantic_operation_candidates, ExpectedChange,
};
use crate::publication::patch::data::{
    PublishedAspectChangePrecision, PublishedAuthoritativePatchEnvelope,
    PublishedAuthoritativeRecordPatch,
};

/// Which rule a committed change broke.
///
/// Every rule compares a record's published semantic changes with its
/// canonical patch operations, so a change that passes them all describes
/// exactly what the commit wrote.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalChangeConsistencyDenialKind {
    /// A record publishes more or fewer semantic changes than its canonical
    /// operations call for.
    SemanticChangeCountMismatch,
    /// A semantic change claims a precision other than exact. Only a
    /// consumer's own admission can widen a change.
    WidenedPrecisionClaimed,
    /// A semantic change matches no remaining canonical operation. With the
    /// counts equal, this is also how an uncovered operation shows: some
    /// other change failed to match it.
    UnjustifiedSemanticChange,
    /// A record's opaque-aspect flag disagrees with its semantic changes.
    OpaquePostureMismatch,
}

/// The work the consistency checks did before they passed or denied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalChangeConsistencyWork {
    records_checked: u64,
    operations_checked: u64,
    expected_changes_materialized: u64,
    semantic_changes_examined: u64,
    semantic_changes_matched: u64,
}

impl RelationalChangeConsistencyWork {
    /// Record patches checked, the denied one included.
    pub const fn records_checked(&self) -> u64 {
        self.records_checked
    }

    /// Canonical operations counted across the checked records.
    pub const fn operations_checked(&self) -> u64 {
        self.operations_checked
    }

    /// Expected changes derived from those operations.
    pub const fn expected_changes_materialized(&self) -> u64 {
        self.expected_changes_materialized
    }

    /// Published semantic changes whose precision was examined.
    pub const fn semantic_changes_examined(&self) -> u64 {
        self.semantic_changes_examined
    }

    /// Published semantic changes matched against expected changes, the
    /// denied one included.
    pub const fn semantic_changes_matched(&self) -> u64 {
        self.semantic_changes_matched
    }
}

/// Why a committed change is not self-consistent, with the work done up to
/// and including the record that broke the rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalChangeConsistencyDenial {
    kind: RelationalChangeConsistencyDenialKind,
    detail: String,
    work: RelationalChangeConsistencyWork,
}

impl RelationalChangeConsistencyDenial {
    /// The rule the change broke.
    pub const fn kind(&self) -> RelationalChangeConsistencyDenialKind {
        self.kind
    }

    /// A description of the breach, naming the offending change where there
    /// is one.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// The work done before the denial.
    pub const fn work(&self) -> RelationalChangeConsistencyWork {
        self.work
    }
}

impl std::fmt::Display for RelationalChangeConsistencyDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for RelationalChangeConsistencyDenial {}

type RuleBreach = (RelationalChangeConsistencyDenialKind, String);

impl PublishedAuthoritativePatchEnvelope {
    /// Check that every record's semantic changes describe exactly its
    /// canonical patch operations: the rules
    /// [`RelationalRuntime::mint_change_receipt`](crate::runtime::RelationalRuntime::mint_change_receipt)
    /// applies before it mints.
    ///
    /// A receipt's patch has already passed. A consumer that holds a patch by
    /// any other route, decoded from storage for instance, checks it here.
    ///
    /// # Errors
    ///
    /// The first rule a record breaks, with the work done up to and including
    /// that record.
    pub fn check_change_consistency(
        &self,
    ) -> Result<RelationalChangeConsistencyWork, RelationalChangeConsistencyDenial> {
        check_change_consistency(self)
    }
}

fn check_change_consistency(
    patch: &PublishedAuthoritativePatchEnvelope,
) -> Result<RelationalChangeConsistencyWork, RelationalChangeConsistencyDenial> {
    let mut work = RelationalChangeConsistencyWork::default();
    for record in &patch.authoritative_record_patches {
        work.records_checked += 1;
        work.operations_checked += record.authoritative_patch.full_grammar_operation_count() as u64;
        if let Err((kind, detail)) = check_record(record, &mut work) {
            return Err(RelationalChangeConsistencyDenial { kind, detail, work });
        }
    }
    Ok(work)
}

fn check_record(
    record: &PublishedAuthoritativeRecordPatch,
    work: &mut RelationalChangeConsistencyWork,
) -> Result<(), RuleBreach> {
    let expected = expected_changes(record);
    work.expected_changes_materialized += expected.len() as u64;
    if expected.len() != record.semantic_changes.len() {
        return Err((
            RelationalChangeConsistencyDenialKind::SemanticChangeCountMismatch,
            "semantic change count did not match canonical patch operations".to_owned(),
        ));
    }
    let contains_opaque = examine_semantic_changes(record, work);
    let mut remaining = expected
        .into_iter()
        .fold(HashMap::new(), |mut counts, change| {
            *counts.entry(change).or_insert(0_usize) += 1;
            counts
        });
    // Equal counts and one expected change consumed per match leave nothing
    // uncovered once every change matches.
    match_semantic_changes(record, &mut remaining, work)?;
    if contains_opaque != record.contains_opaque_aspect {
        return Err((
            RelationalChangeConsistencyDenialKind::OpaquePostureMismatch,
            "opaque aspect posture did not match canonical semantic changes".to_owned(),
        ));
    }
    Ok(())
}

fn examine_semantic_changes(
    record: &PublishedAuthoritativeRecordPatch,
    work: &mut RelationalChangeConsistencyWork,
) -> bool {
    let mut contains_opaque = false;
    for change in &record.semantic_changes {
        work.semantic_changes_examined += 1;
        contains_opaque |= change.kind() == AuthoritativeAspectChangeKind::Opaque;
    }
    contains_opaque
}

fn match_semantic_changes(
    record: &PublishedAuthoritativeRecordPatch,
    remaining: &mut HashMap<ExpectedChange, usize>,
    work: &mut RelationalChangeConsistencyWork,
) -> Result<(), RuleBreach> {
    for change in &record.semantic_changes {
        work.semantic_changes_matched += 1;
        if change.precision() != PublishedAspectChangePrecision::Exact {
            return Err((
                RelationalChangeConsistencyDenialKind::WidenedPrecisionClaimed,
                "published semantic change claimed a widened precision".to_owned(),
            ));
        }
        let matched = semantic_operation_candidates(change, record.structural_change)
            .into_iter()
            .any(|candidate| consume_expected(remaining, candidate));
        if !matched {
            let remaining = remaining
                .iter()
                .map(|(change, count)| (change.clone(), *count))
                .collect::<BTreeMap<_, _>>();
            return Err((
                RelationalChangeConsistencyDenialKind::UnjustifiedSemanticChange,
                format!(
                    "semantic change was not justified by the canonical authoritative patch: change={change:?}; remaining={remaining:?}"
                ),
            ));
        }
    }
    Ok(())
}
