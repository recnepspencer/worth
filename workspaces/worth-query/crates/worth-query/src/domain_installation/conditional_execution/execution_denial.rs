//! Conditional execution keeps Query cache refusal distinct from native Bridge causes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQueryConditionalExecutionDenialKind {
    /// Query cannot retain another evaluation entry while all entries are active.
    EvaluationCacheExhausted,
    /// The exact native Bridge or Signal seam refused its operation.
    Bridge(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
}
