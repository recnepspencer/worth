use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::source_seed::SourceRecomputeSeed;

pub(crate) fn validate_source_seed(
    graph: &SignalGraph,
    seed: &SourceRecomputeSeed,
) -> Result<(), SignalError> {
    if !graph.is_alive(seed.source()) {
        return Err(SignalError::invalid_input(
            "source recompute seed references a non-live source",
        ));
    }
    if seed
        .changed_scopes()
        .iter()
        .any(|scope| scope.path().segments().iter().any(String::is_empty))
    {
        return Err(SignalError::invalid_input(
            "source recompute seed contains an empty partition token",
        ));
    }
    let _ = seed.aspect();
    Ok(())
}
