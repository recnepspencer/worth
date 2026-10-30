mod assembly;
mod diagnostics;
mod operational;
mod report;

pub(crate) use assembly::{EffectRuntimeMetadata, EvaluationEffect, PreviousArtifactWarmSnapshot};
pub use diagnostics::DiagnosticEnvelope;
pub use operational::{DeferralReason, EvaluationVerdict, OperationalEffect, SuppressionReason};
pub(crate) use operational::{
    DependencyInputContext, EffectDependencyInputs, PendingDependencySnapshot,
};
pub(crate) use report::PreparedApplyResult;
pub use report::{AppliedEffectReport, EffectComparison};
