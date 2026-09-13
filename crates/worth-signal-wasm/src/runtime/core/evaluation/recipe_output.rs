use worth_signal::facade::specialist::EvaluationOutput;
use worth_signal::facade::{EvaluationContext, NodeEvaluationResult, OutputChange, SignalError};

use super::super::state::{SharedStore, StoredRecipe};

/// Describe the provisional recipe candidate against the artifact Signal will
/// replace. Recipe initialization and earlier computes are not publication.
pub(super) fn finish_recipe_output(
    view: &EvaluationContext<'_, SharedStore>,
    recipe: &StoredRecipe,
) -> Result<EvaluationOutput, SignalError> {
    let graph = view.graph();
    let previous_version = graph.node_aspect_version(view.node())?;
    // This operational warm record exists exactly when runtime artifact state
    // exists; it is independent of retained diagnostic richness.
    let previous = graph.observe().runtime_artifact_warm(view.node())?;
    let output_change = match previous {
        None => OutputChange::Replaced,
        Some(previous)
            if previous
                .output_identity
                .as_ref()
                .map(|identity| identity.as_str())
                != recipe.output_identity.as_deref() =>
        {
            OutputChange::Replaced
        }
        Some(_) if previous_version == recipe.version => OutputChange::Unchanged,
        Some(_) => OutputChange::Refreshed,
    };
    let mut result =
        NodeEvaluationResult::from_version(recipe.version).with_output_change(output_change);
    if let Some(identity) = &recipe.output_identity {
        result = result.with_output_identity(identity.clone());
    }
    Ok(view.finish(result))
}
