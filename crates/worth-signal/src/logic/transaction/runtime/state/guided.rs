use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::diagnostics::{ReplayView, SynthesizedLineageChain};
use crate::state::{SignalBranchHandle, SignalBranchId};

use super::merge::{
    AspectMergePolicyBinding, AspectMergePolicyName, BranchMergeRequest, BranchMergeRequestDenial,
    BranchMergeRequestScope, BranchMergeResult, BranchMergeStrategy, ConflictIsolationPolicyName,
    ConflictPolicyName, DeletionPolicyName, IdentityMatcherName, LoweredFoundationalMergeRequest,
    MergeBaseStrategyName, MergeStrategyName, NormalizedBranchMergeRequest,
    SignalSelectedAspectRequestEntry, SourceOnlyPolicyName,
};
use super::runtime_state::SignalRuntime;

pub(crate) struct RawPlannedRuntimeMerge<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
    request: crate::logic::transaction::runtime::NormalizedBranchMergeRequest,
    lowered_request: crate::logic::transaction::runtime::LoweredFoundationalMergeRequest,
    plan: crate::logic::transaction::runtime::BranchMergePlan,
}

impl<'a, D, I, E, Ctx, T> RawPlannedRuntimeMerge<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn plan(&self) -> &crate::logic::transaction::runtime::BranchMergePlan {
        &self.plan
    }

    pub fn request(&self) -> &crate::logic::transaction::runtime::BranchMergeRequest {
        self.request.request()
    }

    pub fn normalized_request(
        &self,
    ) -> &crate::logic::transaction::runtime::NormalizedBranchMergeRequest {
        &self.request
    }

    pub fn lowered_request(
        &self,
    ) -> &crate::logic::transaction::runtime::LoweredFoundationalMergeRequest {
        &self.lowered_request
    }

    pub fn execute(self) -> Result<BranchMergeResult, SignalError> {
        self.runtime
            .execute_branch_merge_request_plan(&self.lowered_request, &self.plan)
    }

    pub(crate) fn execute_admitted(
        self,
        source: &crate::branch::AdmittedSignalBranchBasis,
        target: &crate::branch::AdmittedSignalBranchBasis,
    ) -> Result<crate::branch::SignalBranchMergeOutcome, SignalError> {
        self.runtime.execute_admitted_branch_merge_request_plan(
            source,
            target,
            &self.lowered_request,
            &self.plan,
        )
    }
}

pub struct RuntimeHistory<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
}

impl<'a, D, I, E, Ctx, T> RuntimeHistory<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>) -> Self {
        Self { runtime }
    }

    pub fn current_branch(&self) -> SignalBranchHandle {
        self.runtime.current_branch()
    }

    pub fn branches(&self) -> Vec<SignalBranchHandle> {
        self.runtime.known_branches()
    }

    pub fn branch(&self, branch_id: SignalBranchId) -> Option<SignalBranchHandle> {
        self.runtime.branch_handle(branch_id)
    }

    pub fn ancestry(&self, branch_id: SignalBranchId) -> Vec<SignalBranchHandle> {
        self.runtime.branch_ancestry(branch_id)
    }

    pub fn replay_for_branch(&self, branch_id: SignalBranchId) -> ReplayView {
        self.runtime.replay_for_branch(branch_id)
    }

    pub fn replay_for_node(&self, node: crate::data::handle::NodeId) -> ReplayView {
        self.runtime.observe().replay_for_node(node)
    }

    pub fn lineage_for_node(&self, node: crate::data::handle::NodeId) -> SynthesizedLineageChain {
        self.runtime.observe().lineage_chain_for_node(node)
    }

    pub fn latest_lineage(&self) -> crate::diagnostics::lineage::RetainedLineageView<'_> {
        self.runtime.graph().observe().lineage_records()
    }
}

pub(crate) struct RawRuntimeMerge<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
    source: Option<SignalBranchHandle>,
    target: Option<SignalBranchHandle>,
    pub(super) strategy_name: Option<MergeStrategyName>,
    pub(super) strategy_hint: Option<BranchMergeStrategy>,
    pub(super) merge_base_name: Option<MergeBaseStrategyName>,
    pub(super) conflict_policy_name: Option<ConflictPolicyName>,
    pub(super) conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
    pub(super) identity_matcher_name: Option<IdentityMatcherName>,
    pub(super) source_only_policy_name: Option<SourceOnlyPolicyName>,
    pub(super) deletion_policy_name: Option<DeletionPolicyName>,
    pub(super) aspect_policy_bindings: Vec<AspectMergePolicyBinding>,
    pub(super) scope: Option<BranchMergeRequestScope>,
}

impl<'a, D, I, E, Ctx, T> RawRuntimeMerge<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>) -> Self {
        Self {
            runtime,
            source: None,
            target: None,
            strategy_name: None,
            strategy_hint: None,
            merge_base_name: None,
            conflict_policy_name: None,
            identity_matcher_name: None,
            source_only_policy_name: None,
            conflict_isolation_policy_name: None,
            deletion_policy_name: None,
            aspect_policy_bindings: Vec::new(),
            scope: None,
        }
    }

    pub fn from(mut self, branch: SignalBranchHandle) -> Self {
        self.source = Some(branch);
        self
    }

    pub fn into_branch(mut self, branch: SignalBranchHandle) -> Self {
        self.target = Some(branch);
        self
    }

    pub fn into(self, branch: SignalBranchHandle) -> Self {
        self.into_branch(branch)
    }

    pub fn strategy_hint(mut self, strategy: BranchMergeStrategy) -> Self {
        self.strategy_hint = Some(strategy);
        self
    }

    pub fn strategy_named(mut self, strategy_name: impl Into<String>) -> Self {
        self.strategy_name = Some(MergeStrategyName::new(strategy_name));
        self
    }

    pub fn conflict_policy_named(mut self, policy_name: impl Into<String>) -> Self {
        self.conflict_policy_name = Some(ConflictPolicyName::new(policy_name));
        self
    }

    pub fn conflict_isolation_policy_named(mut self, policy_name: impl Into<String>) -> Self {
        self.conflict_isolation_policy_name = Some(ConflictIsolationPolicyName::new(policy_name));
        self
    }

    pub fn merge_base_named(mut self, strategy_name: impl Into<String>) -> Self {
        self.merge_base_name = Some(MergeBaseStrategyName::new(strategy_name));
        self
    }

    pub fn identity_matcher_named(mut self, matcher_name: impl Into<String>) -> Self {
        self.identity_matcher_name = Some(IdentityMatcherName::new(matcher_name));
        self
    }

    pub fn source_only_policy_named(mut self, policy_name: impl Into<String>) -> Self {
        self.source_only_policy_name = Some(SourceOnlyPolicyName::new(policy_name));
        self
    }

    pub fn deletion_policy_named(mut self, policy_name: impl Into<String>) -> Self {
        self.deletion_policy_name = Some(DeletionPolicyName::new(policy_name));
        self
    }

    pub fn aspect_policy_named(mut self, aspect: Aspect, policy_name: impl Into<String>) -> Self {
        self.aspect_policy_bindings
            .push(AspectMergePolicyBinding::new(
                aspect,
                AspectMergePolicyName::new(policy_name),
            ));
        self
    }

    pub fn full_branch(mut self) -> Self {
        self.scope = Some(BranchMergeRequestScope::full_branch());
        self
    }

    pub fn selected_nodes(
        mut self,
        selected_nodes: impl IntoIterator<Item = crate::data::handle::NodeId>,
    ) -> Self {
        self.scope = Some(BranchMergeRequestScope::selected_nodes(selected_nodes));
        self
    }

    pub fn selected_aspects(
        mut self,
        selected_aspects: impl IntoIterator<Item = SignalSelectedAspectRequestEntry>,
    ) -> Self {
        self.scope = Some(BranchMergeRequestScope::selected_aspects(selected_aspects));
        self
    }

    pub fn build_request(&self) -> Result<BranchMergeRequest, SignalError> {
        let source = self
            .source
            .clone()
            .ok_or_else(|| SignalError::invalid_input("merge source branch is required"))?;
        let target = self
            .target
            .clone()
            .ok_or_else(|| SignalError::invalid_input("merge target branch is required"))?;
        Ok(BranchMergeRequest {
            source_branch: source,
            target_branch: target,
            scope: self.scope.clone().unwrap_or_default(),
            strategy_name: self.strategy_name.clone(),
            strategy_hint: self.strategy_hint,
            merge_base_name: self.merge_base_name.clone(),
            conflict_policy_name: self.conflict_policy_name.clone(),
            identity_matcher_name: self.identity_matcher_name.clone(),
            source_only_policy_name: self.source_only_policy_name.clone(),
            deletion_policy_name: self.deletion_policy_name.clone(),
            conflict_isolation_policy_name: self.conflict_isolation_policy_name.clone(),
            aspect_policy_bindings: self.aspect_policy_bindings.clone(),
        })
    }

    pub fn build_normalized_request(&self) -> Result<NormalizedBranchMergeRequest, SignalError> {
        self.build_request().and_then(|request| {
            request
                .normalize()
                .map_err(BranchMergeRequestDenial::into_signal_error)
        })
    }

    pub fn build_lowered_foundational_request(
        self,
    ) -> Result<LoweredFoundationalMergeRequest, SignalError> {
        let request = self.build_normalized_request()?;
        self.runtime.lower_foundational_merge_request(&request)
    }

    pub fn plan(self) -> Result<RawPlannedRuntimeMerge<'a, D, I, E, Ctx, T>, SignalError> {
        let request = self.build_normalized_request()?;
        let lowered_request = self.runtime.lower_foundational_merge_request(&request)?;
        let request = lowered_request.normalized_request().clone();
        let plan = self.runtime.plan_branch_merge_request(&lowered_request)?;
        Ok(RawPlannedRuntimeMerge {
            runtime: self.runtime,
            request,
            lowered_request,
            plan,
        })
    }

    pub fn run(self) -> Result<BranchMergeResult, SignalError> {
        self.plan()?.execute()
    }

    pub fn execute(self) -> Result<BranchMergeResult, SignalError> {
        self.run()
    }
}
