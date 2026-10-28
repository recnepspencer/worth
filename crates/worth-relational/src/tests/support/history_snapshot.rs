//! Explicit execution-funded history assertions for resident or durable fixtures.
use super::*;
use crate::history::data::AuthoritativeArtifactEnvelopeRef;

pub(crate) fn demand_canonical_history_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<Vec<u8>> {
    let mut ids = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| artifact.commit_id())
        .collect::<std::collections::BTreeSet<_>>();
    ids.extend(
        runtime
            .history
            .performed_canonical_envelopes()
            .into_iter()
            .map(|envelope| envelope.commit.commit_id),
    );
    ids.into_iter()
        .map(|id| {
            let envelope = runtime
                .history()
                .demand_commit_envelope(execution, id)
                .expect("fixture history source remains available")
                .expect("catalog identity has a canonical body");
            // Canonical durable history excludes optional transaction diagnostics.
            rmp_serde::to_vec_named(&AuthoritativeArtifactEnvelopeRef::new(&envelope))
                .expect("admitted canonical history encodes")
        })
        .collect()
}
