use worth_foundational::JsonCompatibilityLoweringOutcome;

use crate::certification::ordinary_context::compatibility::json_lowering_fixtures::source_for;

#[test]
fn json_lowering_outcome_preserves_non_success_categories() {
    let deferred: JsonCompatibilityLoweringOutcome<()> = worth_proof::TransitionOutcome::deferred(
        worth_foundational::JsonCompatibilityLoweringDeferred::new(
            source_for("task.summary"),
            "schema not loaded",
        ),
    );
    let stale: JsonCompatibilityLoweringOutcome<()> = worth_proof::TransitionOutcome::stale(
        worth_foundational::JsonCompatibilityLoweringStale::new(
            source_for("task.summary"),
            "schema revision stale",
        ),
    );
    let rebind: JsonCompatibilityLoweringOutcome<()> =
        worth_proof::TransitionOutcome::rebind_required(
            worth_foundational::JsonCompatibilityRebindRequired::new(
                source_for("task.summary"),
                "aspect key rebound",
            ),
        );
    let failed: JsonCompatibilityLoweringOutcome<()> = worth_proof::TransitionOutcome::failed(
        worth_foundational::JsonCompatibilityLoweringFailure::new(
            source_for("task.summary"),
            "transport corrupt",
        ),
    );

    assert!(matches!(
        deferred,
        worth_proof::TransitionOutcome::Deferred(_)
    ));
    assert!(matches!(stale, worth_proof::TransitionOutcome::Stale(_)));
    assert!(matches!(
        rebind,
        worth_proof::TransitionOutcome::RebindRequired(_)
    ));
    assert!(matches!(failed, worth_proof::TransitionOutcome::Failed(_)));
}
