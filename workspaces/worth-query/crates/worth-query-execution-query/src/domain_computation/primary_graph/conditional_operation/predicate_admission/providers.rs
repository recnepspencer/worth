use super::*;

/// Query's governed re-entry input for one conditional computation.
pub struct QueryConditionalComputeContext {
    attempt: u64,
    fallback: QueryConditionalFallbackVersion,
}

/// The output version used when no host output-version provider is installed.
#[derive(Clone, Copy)]
enum QueryConditionalFallbackVersion {
    /// The attempt itself, compared exactly as supplied.
    Attempt,
    /// One past the node's version before the attempt: each evaluation is a
    /// fresh occurrence, never an unchanged recompute.
    PastBaseline,
}

impl QueryConditionalComputeContext {
    /// The attempt is the output version unless a host provider projects one.
    pub const fn exact(attempt: u64) -> Self {
        Self {
            attempt,
            fallback: QueryConditionalFallbackVersion::Attempt,
        }
    }

    /// A fresh occurrence (a due wake or its reconsideration): unless a host
    /// provider projects a version from `attempt`, the output advances past
    /// the node's version before the attempt.
    pub const fn fresh(attempt: u64) -> Self {
        Self {
            attempt,
            fallback: QueryConditionalFallbackVersion::PastBaseline,
        }
    }
}

/// The node output a Query compute provider returns: a host provider's
/// version for the true attempt, else the context's fallback version.
pub(super) fn computed_output(
    context: &dyn std::any::Any,
    output_version_before: Option<u64>,
    host_version: impl FnOnce(u64) -> Option<Result<u64, String>>,
) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
    let context = context
        .downcast_ref::<QueryConditionalComputeContext>()
        .ok_or_else(|| {
            "conditional execution lacked Query's governed re-entry context".to_owned()
        })?;
    let version = match host_version(context.attempt).transpose()? {
        Some(version) => version,
        None => match (context.fallback, output_version_before) {
            (QueryConditionalFallbackVersion::PastBaseline, Some(before)) => before
                .checked_add(1)
                .ok_or_else(|| "conditional output version space is exhausted".to_owned())?,
            (QueryConditionalFallbackVersion::Attempt, _)
            | (QueryConditionalFallbackVersion::PastBaseline, None) => context.attempt,
        },
    };
    Ok(worth_signal::facade::NodeEvaluationResult::from_version(
        worth_signal::facade::AspectVersion::from_updates([(
            worth_signal::facade::Aspect::new(0),
            version,
        )]),
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QueryConditionalComputeSemanticContract(pub(super) Arc<str>);

pub(super) struct QueryConditionalComputeProvider<Node> {
    pub(super) semantics: QueryConditionalComputeSemanticContract,
    pub(super) output_version:
        Option<Arc<dyn WorthQueryHostConditionalOutputVersionProvider<Node>>>,
}

impl<Node: 'static> BridgeConditionalProviderSemantics for QueryConditionalComputeProvider<Node> {
    type SemanticContract = QueryConditionalComputeSemanticContract;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.semantics.clone()
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        let mut provider = vec![
            worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                self.semantics.0.as_ref(),
            ),
        ];
        if let Some(output_version) = self.output_version.as_ref() {
            provider.push(
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    output_version.as_ref(),
                ),
            );
            provider.push(
                output_version
                    .retained_heap_bytes()
                    .map_err(|_| {
                        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow
                    })?
                    .bytes(),
            );
        }
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            provider,
            [],
        )
    }
}

impl<Node: 'static> BridgeConditionalComputeProvider for QueryConditionalComputeProvider<Node> {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        computed_output(context, None, |attempt| self.host_version(attempt))
    }

    fn compute_from_baseline(
        &self,
        context: &mut dyn std::any::Any,
        output_version_before: u64,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        computed_output(context, Some(output_version_before), |attempt| {
            self.host_version(attempt)
        })
    }
}

impl<Node: 'static> QueryConditionalComputeProvider<Node> {
    fn host_version(&self, attempt: u64) -> Option<Result<u64, String>> {
        self.output_version.as_ref().map(|provider| {
            provider
                .output_version(attempt)
                .map_err(|failure| failure.detail().to_owned())
        })
    }
}

pub(super) struct QueryHostOutputComparator<Node> {
    pub(super) identity: Arc<str>,
    pub(super) provider: Arc<dyn WorthQueryHostConditionalOutputComparatorProvider<Node>>,
}

impl<Node: 'static> BridgeConditionalProviderSemantics for QueryHostOutputComparator<Node> {
    type SemanticContract = Arc<str>;

    fn semantic_contract(&self) -> Self::SemanticContract {
        Arc::clone(&self.identity)
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.identity.as_ref()),
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.provider.as_ref()),
                self.provider.retained_heap_bytes().map_err(|_| worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow)?.bytes(),
            ],
            [],
        )
    }
}

impl<Node: 'static> worth_runtime_bridge::facade::BridgeConditionalComparatorProvider
    for QueryHostOutputComparator<Node>
{
    fn has_meaningful_change(
        &self,
        _aspect: worth_signal::facade::Aspect,
        cached: u64,
        current: u64,
    ) -> Result<bool, String> {
        self.provider
            .has_meaningful_change(cached, current)
            .map_err(|failure| failure.detail().to_owned())
    }
}

#[cfg(test)]
#[path = "providers_tests.rs"]
mod tests;
