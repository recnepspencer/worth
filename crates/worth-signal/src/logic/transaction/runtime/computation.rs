mod commit_evaluation;

use crate::data::aspect::AspectVersion;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::NodeContract;
use crate::data::output::{
    ComputationFamily, ComputationKey, KeyedComputation, PartitionSubscription, StructuralMemoKey,
};
use crate::data::proof::PartitionScopeSet;
use crate::data::reuse::PersistentCorrespondenceEvidence;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationRequestMode, IntoEvaluationOutput};
use crate::logic::planner::ExecutionReport;

use super::state::SignalRuntime;
use super::transaction::SignalTransaction;

pub struct Recipe<T, F> {
    pub family: ComputationFamily,
    pub contract: NodeContract,
    pub tier: T,
    pub comparator: VersionComparatorPolicy,
    pub evaluator: F,
}

impl<T, F> Recipe<T, F> {
    pub fn new(family: impl Into<ComputationFamily>, tier: T, evaluator: F) -> Self {
        Self {
            family: family.into(),
            contract: NodeContract::wildcard(),
            tier,
            comparator: VersionComparatorPolicy::Exact,
            evaluator,
        }
    }

    pub fn with_contract(mut self, contract: NodeContract) -> Self {
        self.contract = contract;
        self
    }

    pub fn with_tier(mut self, tier: T) -> Self {
        self.tier = tier;
        self
    }

    pub fn with_comparator(mut self, comparator: VersionComparatorPolicy) -> Self {
        self.contract = self.contract.with_comparator_override(&comparator);
        self.comparator = comparator;
        self
    }

    pub fn reads(mut self, reads: impl Into<crate::data::aspect::AspectMask>) -> Self {
        self.contract = self.contract.with_reads(reads);
        self
    }

    pub fn produces(mut self, produces: impl Into<crate::data::aspect::AspectMask>) -> Self {
        self.contract = self.contract.with_produces(produces);
        self
    }

    pub fn partition_scope(mut self, partition_scope: impl Into<PartitionSubscription>) -> Self {
        self.contract = self.contract.with_partition_scope(partition_scope);
        self
    }

    pub fn partition_scopes(
        mut self,
        partition_scopes: impl IntoIterator<Item = PartitionSubscription>,
    ) -> Self {
        self.contract = self.contract.with_partition_scopes(partition_scopes);
        self
    }

    pub fn required_context(
        mut self,
        required_context: crate::data::node::ContextRequirement,
    ) -> Self {
        self.contract = self.contract.with_required_context(required_context);
        self
    }
}

pub struct DefinedComputation<T, F> {
    family: ComputationFamily,
    contract: NodeContract,
    tier: T,
    comparator: VersionComparatorPolicy,
    evaluator: F,
}

pub struct DefinedKeyedComputation<'a, T, F> {
    definition: &'a DefinedComputation<T, F>,
    key: ComputationKey,
}

impl<T: Copy, F> DefinedComputation<T, F> {
    pub(crate) fn from_recipe(spec: Recipe<T, F>) -> Self {
        Self {
            family: spec.family,
            contract: spec.contract,
            tier: spec.tier,
            comparator: spec.comparator,
            evaluator: spec.evaluator,
        }
    }

    pub fn family(&self) -> &ComputationFamily {
        &self.family
    }

    pub fn contract(&self) -> &NodeContract {
        &self.contract
    }

    pub fn tier(&self) -> T {
        self.tier
    }

    pub fn comparator(&self) -> &VersionComparatorPolicy {
        &self.comparator
    }

    pub fn keyed(&self, key: impl Into<ComputationKey>) -> DefinedKeyedComputation<'_, T, F> {
        DefinedKeyedComputation {
            definition: self,
            key: key.into(),
        }
    }
}

impl<'a, T: Copy, F> DefinedKeyedComputation<'a, T, F> {
    pub fn key(&self) -> &ComputationKey {
        &self.key
    }

    pub fn family(&self) -> &ComputationFamily {
        self.definition.family()
    }

    pub fn node<D, I, E, Ctx>(&self, runtime: &mut SignalRuntime<D, I, E, Ctx, T>) -> NodeId
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Ord,
    {
        runtime.config.resolve_defined_node(
            &mut runtime.graph,
            self.definition.family(),
            self.key.clone(),
        )
    }

    pub fn node_in_transaction<D, I, E, Ctx>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
    ) -> NodeId
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        T: Ord,
    {
        tx.resolve_defined_node(self.definition.family(), self.key.clone())
    }

    pub fn metadata(&self) -> KeyedComputation {
        KeyedComputation::new(self.definition.family().clone(), self.key.clone())
    }

    pub fn memoized(&self, memo_key: impl Into<StructuralMemoKey>) -> KeyedComputation {
        self.metadata().with_memo_key(memo_key)
    }
}

impl<'a, T, F> DefinedKeyedComputation<'a, T, F>
where
    T: Copy + Ord,
{
    pub fn execute<D, I, E, Ctx, O>(
        &self,
        runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
        runtime_ctx: &Ctx,
    ) -> Result<ExecutionReport, SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node(runtime);
        runtime.evaluate_with_plan(
            node,
            runtime_ctx,
            &self.definition.evaluator,
            EvaluationRequestMode::Default,
        )
    }

    pub fn run<D, I, E, Ctx, O>(
        &self,
        runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
        runtime_ctx: &Ctx,
    ) -> Result<ExecutionReport, SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.execute(runtime, runtime_ctx)
    }

    pub fn read<D, I, E, Ctx, O>(
        &self,
        runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
        runtime_ctx: &Ctx,
    ) -> Result<AspectVersion, SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node(runtime);
        runtime.read(node, runtime_ctx, &self.definition.evaluator)
    }

    pub fn evaluate<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed(node, &self.metadata(), &self.definition.evaluator)
    }

    pub fn evaluate_memoized<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        memo_key: impl Into<StructuralMemoKey>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed(node, &self.memoized(memo_key), &self.definition.evaluator)
    }

    pub fn evaluate_cross_identity<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        source_key: impl Into<ComputationKey>,
        memo_key: impl Into<StructuralMemoKey>,
        correspondence: impl Into<String>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed_cross_identity(
            node,
            &self.memoized(memo_key),
            &self.definition.evaluator,
            source_key.into(),
            PersistentCorrespondenceEvidence::host_supplied_key(correspondence),
        )
    }

    pub fn evaluate_cross_identity_with_contract_basis<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        source_key: impl Into<ComputationKey>,
        memo_key: impl Into<StructuralMemoKey>,
        basis: impl Into<String>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed_cross_identity(
            node,
            &self.memoized(memo_key),
            &self.definition.evaluator,
            source_key.into(),
            PersistentCorrespondenceEvidence::contract_declared_basis(basis),
        )
    }

    pub fn evaluate_cross_identity_with_lineage_mapping<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        source_key: impl Into<ComputationKey>,
        memo_key: impl Into<StructuralMemoKey>,
        mapping: impl Into<String>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed_cross_identity(
            node,
            &self.memoized(memo_key),
            &self.definition.evaluator,
            source_key.into(),
            PersistentCorrespondenceEvidence::lineage_backed_mapping(mapping),
        )
    }

    pub fn evaluate_cross_identity_with_region_identity<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        source_key: impl Into<ComputationKey>,
        memo_key: impl Into<StructuralMemoKey>,
        region_identity: impl Into<String>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed_cross_identity(
            node,
            &self.memoized(memo_key),
            &self.definition.evaluator,
            source_key.into(),
            PersistentCorrespondenceEvidence::region_identity_basis(region_identity),
        )
    }

    pub fn evaluate_partial_splice<D, I, E, Ctx, O>(
        &self,
        tx: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        memo_key: impl Into<StructuralMemoKey>,
        composition_regions: impl IntoIterator<Item = PartitionSubscription>,
    ) -> Result<(), SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let node = self.node_in_transaction(tx);
        tx.evaluate_keyed_partial_splice(
            node,
            &self.memoized(memo_key),
            &self.definition.evaluator,
            PartitionScopeSet::from(composition_regions.into_iter().collect::<Vec<_>>()),
        )
    }
}
