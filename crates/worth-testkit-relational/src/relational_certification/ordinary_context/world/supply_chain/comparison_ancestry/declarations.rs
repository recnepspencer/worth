pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonFailure, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel};

pub fn compare_ancestry(
    expected: &ExpectedSupplyChainObservation,
    observed: &ObservedSupplyChainState,
) -> Result<(), ComparisonFailure> {
    if expected.ancestry.branch != observed.branch {
        let mismatch = if observed.branch == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating {
            crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::FloatingBranchSelection(observed.branch)
        } else {
            crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::SiblingFactLeak {
                expected: expected.ancestry.branch,
                observed: observed.branch,
            }
        };
        return Err(ComparisonFailure { mismatch });
    }
    if expected.ancestry.parent != observed.parent {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::WrongAncestry {
                expected: expected.ancestry.parent.unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
                observed: observed.parent,
            },
        });
    }
    if expected.ancestry.lineage != observed.lineage {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::WrongAncestry {
                expected: expected
                    .ancestry
                    .lineage
                    .last()
                    .copied()
                    .unwrap_or(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
                observed: observed.lineage.last().copied(),
            },
        });
    }
    if expected.ancestry.accepted != observed.accepted {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::AcceptedDeltaOrder {
                expected: expected.ancestry.accepted.clone(),
                observed: observed.accepted.clone(),
            },
        });
    }
    if expected.ancestry.history != observed.history {
        return Err(ComparisonFailure {
            mismatch: crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::AcceptedHistory {
                expected: expected.ancestry.history.clone(),
                observed: observed.history.clone(),
            },
        });
    }
    Ok(())
}