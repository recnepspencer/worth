pub(crate) use super::comparison_state::{
    ComparisonFailure, ComparisonMismatch, ObservedSupplyChainState,
};
pub(crate) use super::expected_observation::ExpectedSupplyChainObservation;
pub(crate) use super::semantic_key::BranchLabel;

pub(crate) fn compare_ancestry(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    if expected.ancestry.branch != observed.branch {
        let mismatch = if observed.branch == BranchLabel::Operating {
            ComparisonMismatch::FloatingBranchSelection(observed.branch)
        } else {
            ComparisonMismatch::SiblingFactLeak {
                expected: expected.ancestry.branch,
                observed: observed.branch,
            }
        };
        return Err(ComparisonFailure { mismatch });
    }
    if expected.ancestry.parent != observed.parent {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::WrongAncestry {
                expected: expected.ancestry.parent.unwrap_or(BranchLabel::Operating),
                observed: observed.parent,
            },
        });
    }
    if expected.ancestry.lineage != observed.lineage {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::WrongAncestry {
                expected: expected
                    .ancestry
                    .lineage
                    .last()
                    .copied()
                    .unwrap_or(BranchLabel::Operating),
                observed: observed.lineage.last().copied(),
            },
        });
    }
    if expected.ancestry.accepted != observed.accepted {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::AcceptedDeltaOrder {
                expected: expected.ancestry.accepted.clone(),
                observed: observed.accepted.clone(),
            },
        });
    }
    if expected.ancestry.history != observed.history {
        return Err(ComparisonFailure {
            mismatch: ComparisonMismatch::AcceptedHistory {
                expected: expected.ancestry.history.clone(),
                observed: observed.history.clone(),
            },
        });
    }
    Ok(())
}
