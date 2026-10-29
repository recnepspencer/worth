use worth_foundational::{
    JsonCompatibilityLoweringDeferred, JsonCompatibilityLoweringFailure,
    JsonCompatibilityLoweringOutcome, JsonCompatibilityLoweringStale,
    JsonCompatibilityRebindRequired,
};
use worth_proof::TransitionOutcome;

use super::json_lowering_fixtures::source_for;

#[test]
fn json_lowering_outcome_preserves_non_success_categories() {
    let deferred: JsonCompatibilityLoweringOutcome<()> = TransitionOutcome::deferred(
        JsonCompatibilityLoweringDeferred::new(source_for("task.summary"), "schema not loaded"),
    );
    let stale: JsonCompatibilityLoweringOutcome<()> = TransitionOutcome::stale(
        JsonCompatibilityLoweringStale::new(source_for("task.summary"), "schema revision stale"),
    );
    let rebind: JsonCompatibilityLoweringOutcome<()> = TransitionOutcome::rebind_required(
        JsonCompatibilityRebindRequired::new(source_for("task.summary"), "aspect key rebound"),
    );
    let failed: JsonCompatibilityLoweringOutcome<()> = TransitionOutcome::failed(
        JsonCompatibilityLoweringFailure::new(source_for("task.summary"), "transport corrupt"),
    );

    assert!(matches!(deferred, TransitionOutcome::Deferred(_)));
    assert!(matches!(stale, TransitionOutcome::Stale(_)));
    assert!(matches!(rebind, TransitionOutcome::RebindRequired(_)));
    assert!(matches!(failed, TransitionOutcome::Failed(_)));
}
