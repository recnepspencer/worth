//! Installation of already-materialized node artifacts.
use crate::data::node::{NodeColdData, NodeWarmData};
use crate::data::trace::ArtifactWriteDelta;

pub(in crate::data::graph) fn apply_artifact_write(
    warm: &mut NodeWarmData,
    cold: &mut Option<Box<NodeColdData>>,
    delta: ArtifactWriteDelta,
) -> bool {
    warm.runtime_artifact_state = delta.runtime;
    let retained_present = delta.retained.is_some();
    if retained_present {
        cold.get_or_insert_with(|| Box::new(NodeColdData::default()))
            .retained_artifact = delta.retained;
    } else if let Some(cold) = cold.as_mut() {
        cold.retained_artifact = None;
    }
    trim_empty_cold(cold);
    retained_present
}

pub(super) fn trim_empty_cold(cold: &mut Option<Box<NodeColdData>>) {
    if cold.as_ref().is_some_and(|cold| {
        cold.retained_artifact.is_none()
            && cold.causality.is_none()
            && cold.execution_trace.is_none()
    }) {
        *cold = None;
    }
}
