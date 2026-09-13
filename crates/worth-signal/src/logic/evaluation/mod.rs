mod condition;
mod effect;
mod engine;
mod output;
mod reuse;

pub use condition::{
    ConditionEvaluationContext, ConditionResolver, DefaultConditionResolver, EvaluationRequestMode,
    TemporalConditionResolver,
};
pub use effect::{
    AppliedEffectReport, DeferralReason, DiagnosticEnvelope, EvaluationVerdict, OperationalEffect,
    SuppressionReason,
};
pub(crate) use effect::{
    DependencyInputContext, EffectComparison, EffectDependencyInputs, EffectRuntimeMetadata,
    EvaluationEffect, PendingDependencySnapshot, PreparedApplyResult, PreviousArtifactWarmSnapshot,
};
pub(crate) use engine::apply_prepared_evaluation_after_dependencies_with_policy;
pub(crate) use engine::apply_prepared_evaluation_with_policy;
pub(crate) use engine::collect_effect_dependency_inputs_iter;
pub use engine::EvaluationExecutionMetadata;
pub(crate) use engine::EvaluationWork;
#[cfg(feature = "parallel")]
pub(crate) use engine::{
    build_prepared_apply_commit_packet, record_reuse_rejection_telemetry, ApplyCommitBuildError,
};
pub use output::{EvaluationOutput, IntoEvaluationOutput};
#[cfg(feature = "parallel")]
pub(crate) use reuse::resolve_reuse_boundary_authority_with_policy;
#[cfg(feature = "parallel")]
pub(crate) use reuse::resolve_reuse_boundary_context_with_policy;
pub(crate) use reuse::{
    certify_reuse_decision, resolve_prepared_reuse_decision, resolve_reuse_boundary_authority,
    resolve_reuse_boundary_context,
};
