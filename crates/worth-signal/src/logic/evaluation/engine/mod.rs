mod apply;
mod metadata;
mod prepared_apply;
mod work;
pub(crate) use work::EvaluationWork;

pub use metadata::EvaluationExecutionMetadata;
pub(crate) use prepared_apply::apply_prepared_evaluation_after_dependencies_with_policy;
pub(crate) use prepared_apply::apply_prepared_evaluation_with_policy;
#[cfg(feature = "parallel")]
pub(crate) use prepared_apply::{
    build_prepared_apply_commit_packet, record_reuse_rejection_telemetry, ApplyCommitBuildError,
};

pub(crate) fn collect_effect_dependency_inputs_iter<I>(
    graph: &mut crate::data::graph::SignalGraph,
    nodes: I,
) -> Result<Vec<crate::logic::evaluation::EffectDependencyInputs>, crate::data::error::SignalError>
where
    I: IntoIterator<Item = crate::data::handle::NodeId>,
{
    apply::collect_effect_dependency_inputs_iter(graph, nodes)
}
