use std::{marker::PhantomData, sync::Arc};

use worth_query_installation::facade::{
    WorthQueryConditionalDependencyObservation, WorthQueryConditionalObservationTruthBasis,
    WorthQueryConditionalObservationView, WorthQueryConditionalObservedValue,
    WorthQueryHostConditionalPredicateProvider, WorthQueryHostPredicateDecision,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalProviderSemantics, BridgeConditionalResolverContext,
    BridgeConditionalWakeProvider,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryTemporalPredicateSemanticContract {
    node_authority: Arc<str>,
    provider_identity: &'static str,
}

pub(in crate::domain_computation::primary_graph) struct QueryTemporalPredicateProvider<
    Node,
    Provider,
> {
    provider: Arc<Provider>,
    semantics: QueryTemporalPredicateSemanticContract,
    marker: PhantomData<fn() -> Node>,
}

impl<Node, Provider> QueryTemporalPredicateProvider<Node, Provider>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        provider: Arc<Provider>,
        node_authority: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            provider,
            semantics: QueryTemporalPredicateSemanticContract {
                node_authority: node_authority.into(),
                provider_identity: Provider::SEMANTIC_IDENTITY,
            },
            marker: PhantomData,
        }
    }

    fn evaluate(
        &self,
        context: &BridgeConditionalResolverContext,
    ) -> Result<WorthQueryHostPredicateDecision, String> {
        let branch = context.truth_branch_identity().ok_or_else(|| {
            "conditional predicate observation did not retain its truth branch".to_string()
        })?;
        let observations = context
            .observations()
            .iter()
            .map(|observation| {
                WorthQueryConditionalDependencyObservation::from_runtime_observation(
                    observation.dependency_ordinal(),
                    observation
                        .previous()
                        .map(|artifact| {
                            WorthQueryConditionalObservedValue::Present(
                                worth_query_installation::facade::WorthQueryConditionalProjectedValue::from_runtime_projection(
                                    artifact,
                                    observation.projection_mask(),
                                ),
                            )
                        })
                        .unwrap_or(WorthQueryConditionalObservedValue::Absent),
                    observation
                        .current()
                        .map(|artifact| {
                            WorthQueryConditionalObservedValue::Present(
                                worth_query_installation::facade::WorthQueryConditionalProjectedValue::from_runtime_projection(
                                    artifact,
                                    observation.projection_mask(),
                                ),
                            )
                        })
                        .unwrap_or(WorthQueryConditionalObservedValue::Absent),
                )
            })
            .collect::<Vec<_>>();
        let basis = WorthQueryConditionalObservationTruthBasis::from_runtime_truth(
            branch,
            context.truth_snapshot_identity(),
        );
        let view =
            WorthQueryConditionalObservationView::from_runtime_observations(basis, &observations);
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.provider.evaluate(view)
        })) {
            Ok(Ok(decision)) => Ok(decision),
            Ok(Err(failure)) => Err(format!(
                "host conditional predicate failed ({:?}): {}",
                failure.kind(),
                failure.detail()
            )),
            Err(_) => Err("host conditional predicate panicked".to_string()),
        }
    }
}

impl<Node, Provider> BridgeConditionalProviderSemantics
    for QueryTemporalPredicateProvider<Node, Provider>
where
    Node: 'static,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    type SemanticContract = QueryTemporalPredicateSemanticContract;

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
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.provider.as_ref()),
                self.provider.retained_heap_bytes().map_err(|_| worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow)?.bytes(),
                worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(self.semantics.node_authority.as_ref()),
            ],
            [],
        )
    }
}

impl<Node, Provider> BridgeConditionalWakeProvider
    for QueryTemporalPredicateProvider<Node, Provider>
where
    Node: 'static,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        self.evaluate(&context).map(|decision| match decision {
            WorthQueryHostPredicateDecision::Satisfied => {
                worth_signal::facade::InstalledSignalConditionDecision::Eligible
            }
            WorthQueryHostPredicateDecision::Unsatisfied => {
                worth_signal::facade::InstalledSignalConditionDecision::Suppressed
            }
        })
    }
}
