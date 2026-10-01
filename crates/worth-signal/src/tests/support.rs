use crate::data::output::IntoNodeEvaluationResult;
use crate::facade::advanced::{
    ComparatorPolicyResolver, ConditionResolver, DefaultComparatorPolicyResolver,
    DefaultComparatorResolver, DefaultConditionResolver, EvaluationContext, EvaluationOutput,
    EvaluationRequestMode, VersionComparatorPolicy, VersionComparatorResolver,
};
use crate::facade::integration::NodeContract;
use crate::facade::runtime::{ComputationFamily, DefinedComputation, Recipe, SignalRuntime};
use crate::facade::{
    Aspect, AspectMask, AspectVersion, DependencyBatchEdit, DependencyEdge, NodeId, PartitionToken,
    SignalError, SignalGraph,
};
use crate::logic::planner::{
    build_evaluation_plan_with_policy_resolver, execute_plan_with_policy_and_condition,
};
use std::collections::BTreeMap;
use std::ops::DerefMut;

pub const ASPECT_A: Aspect = Aspect::new(0);
pub const ASPECT_B: Aspect = Aspect::new(1);

pub fn mask_a() -> AspectMask {
    AspectMask::from_aspect(ASPECT_A)
}

pub fn mask_b() -> AspectMask {
    AspectMask::from_aspect(ASPECT_B)
}

pub fn version_ab(a: u64, b: u64) -> AspectVersion {
    AspectVersion::from_updates([(ASPECT_A, a), (ASPECT_B, b)])
}

pub struct DependencyBatchBuilder<G>
where
    G: DerefMut<Target = SignalGraph>,
{
    graph: G,
    pending: BTreeMap<NodeId, Vec<DependencyEdge>>,
}

impl<G> DependencyBatchBuilder<G>
where
    G: DerefMut<Target = SignalGraph>,
{
    pub fn new(graph: G) -> Self {
        Self {
            graph,
            pending: BTreeMap::new(),
        }
    }

    pub fn append_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<&mut Self, SignalError> {
        self.dependencies_for(downstream)?
            .push(DependencyEdge::new(upstream, aspect));
        Ok(self)
    }

    pub fn append_partition_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
    ) -> Result<&mut Self, SignalError> {
        self.dependencies_for(downstream)?
            .push(DependencyEdge::whole_partition(upstream, aspect, partition));
        Ok(self)
    }

    pub fn append_partition_detail_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Result<&mut Self, SignalError> {
        self.dependencies_for(downstream)?
            .push(DependencyEdge::partition_detail(
                upstream, aspect, partition, detail,
            ));
        Ok(self)
    }

    pub fn commit(mut self) -> Result<(), SignalError> {
        self.graph
            .deref_mut()
            .apply_dependency_batch_edit(&DependencyBatchEdit::from_pairs(std::mem::take(
                &mut self.pending,
            )))
    }

    fn dependencies_for(&mut self, node: NodeId) -> Result<&mut Vec<DependencyEdge>, SignalError> {
        if !self.pending.contains_key(&node) {
            self.pending
                .insert(node, self.graph.deref_mut().dependencies_of(node)?.to_vec());
        }
        Ok(self
            .pending
            .get_mut(&node)
            .expect("pending dependency batch should contain node"))
    }
}

pub trait GraphDependencyBatchExt {
    fn append_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError>;

    fn append_partition_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
    ) -> Result<(), SignalError>;

    fn append_partition_detail_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Result<(), SignalError>;

    fn drop_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError>;

    fn rewire_dependency(
        &mut self,
        downstream: NodeId,
        old_upstream: NodeId,
        new_upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError>;
}

impl GraphDependencyBatchExt for SignalGraph {
    fn append_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError> {
        self.append_simple_dependency_edge(downstream, upstream, aspect)
            .map(|_| ())
    }

    fn append_partition_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
    ) -> Result<(), SignalError> {
        self.edit_dependencies(downstream, |dependencies| {
            dependencies.push(DependencyEdge::whole_partition(upstream, aspect, partition));
        })
    }

    fn append_partition_detail_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Result<(), SignalError> {
        self.edit_dependencies(downstream, |dependencies| {
            dependencies.push(DependencyEdge::partition_detail(
                upstream, aspect, partition, detail,
            ));
        })
    }

    fn drop_dependency(
        &mut self,
        downstream: NodeId,
        upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError> {
        self.drop_simple_dependency_edge(downstream, upstream, aspect)
            .map(|_| ())
    }

    fn rewire_dependency(
        &mut self,
        downstream: NodeId,
        old_upstream: NodeId,
        new_upstream: NodeId,
        aspect: Aspect,
    ) -> Result<(), SignalError> {
        self.rewire_simple_dependency_edge(downstream, old_upstream, new_upstream, aspect)
            .map(|_| ())
    }
}

pub(crate) fn evaluate<F, O>(
    mut graph: impl DerefMut<Target = SignalGraph>,
    node: NodeId,
    compute: &mut F,
) -> Result<(), SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: IntoNodeEvaluationResult,
{
    let mut comparator = DefaultComparatorResolver;
    let mut condition = DefaultConditionResolver;
    evaluate_with_resolvers(
        graph.deref_mut(),
        node,
        compute,
        &mut comparator,
        &mut condition,
        EvaluationRequestMode::Default,
    )
}

pub(crate) fn evaluate_on_demand<F, O>(
    mut graph: impl DerefMut<Target = SignalGraph>,
    node: NodeId,
    compute: &mut F,
) -> Result<(), SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: IntoNodeEvaluationResult,
{
    let mut comparator = DefaultComparatorResolver;
    let mut condition = DefaultConditionResolver;
    evaluate_with_resolvers(
        graph.deref_mut(),
        node,
        compute,
        &mut comparator,
        &mut condition,
        EvaluationRequestMode::ForceOnDemand,
    )
}

pub(crate) fn evaluate_with_resolver<F, O, R>(
    mut graph: impl DerefMut<Target = SignalGraph>,
    node: NodeId,
    compute: &mut F,
    resolver: &mut R,
) -> Result<(), SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: IntoNodeEvaluationResult,
    R: VersionComparatorResolver,
{
    let mut condition = DefaultConditionResolver;
    evaluate_with_resolvers(
        graph.deref_mut(),
        node,
        compute,
        resolver,
        &mut condition,
        EvaluationRequestMode::Default,
    )
}

pub(crate) fn evaluate_with_resolvers<F, O, R, C>(
    mut graph: impl DerefMut<Target = SignalGraph>,
    node: NodeId,
    compute: &mut F,
    comparator_resolver: &mut R,
    condition_resolver: &mut C,
    request_mode: EvaluationRequestMode,
) -> Result<(), SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: IntoNodeEvaluationResult,
    R: VersionComparatorResolver,
    C: ConditionResolver,
{
    let mut policy = DefaultComparatorPolicyResolver {
        fallback: VersionComparatorPolicy::Exact,
        custom: comparator_resolver,
    };
    evaluate_with_policy_and_condition_resolvers(
        graph.deref_mut(),
        node,
        compute,
        &mut policy,
        condition_resolver,
        request_mode,
    )
}

pub(crate) fn evaluate_with_policy_and_condition_resolvers<F, O, R, C>(
    mut graph: impl DerefMut<Target = SignalGraph>,
    node: NodeId,
    compute: &mut F,
    comparator_resolver: &mut R,
    condition_resolver: &mut C,
    request_mode: EvaluationRequestMode,
) -> Result<(), SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: IntoNodeEvaluationResult,
    R: ComparatorPolicyResolver,
    C: ConditionResolver,
{
    let graph = graph.deref_mut();
    let plan = build_evaluation_plan_with_policy_resolver(
        graph,
        &[node],
        request_mode,
        comparator_resolver,
    )?;
    execute_plan_with_policy_and_condition(
        graph,
        &plan,
        compute,
        comparator_resolver,
        condition_resolver,
        None,
    )?;
    Ok(())
}

fn unsupported_keyed_evaluator(
    _ctx: &mut EvaluationContext<'_, ()>,
) -> Result<EvaluationOutput, SignalError> {
    Err(SignalError::internal(
        "test helper computation definition should not use its built-in evaluator",
    ))
}

pub(crate) fn define_keyed_computation<D, I, E, Ctx, T>(
    runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
    family: impl Into<ComputationFamily>,
    tier: T,
) -> DefinedComputation<
    T,
    fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError>,
>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime
        .define(Recipe {
            family: family.into(),
            contract: NodeContract::wildcard(),
            tier,
            comparator: VersionComparatorPolicy::Exact,
            evaluator: unsupported_keyed_evaluator
                as fn(&mut EvaluationContext<'_, ()>) -> Result<EvaluationOutput, SignalError>,
        })
        .expect("test keyed computation should define cleanly")
}
