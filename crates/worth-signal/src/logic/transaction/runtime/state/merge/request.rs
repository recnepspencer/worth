mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::transaction::canonical_digest;
use crate::state::SignalBranchHandle;

use super::aspect_policy_registry::AspectMergePolicyBinding;
use super::conflict_isolation_registry::ConflictIsolationPolicyName;
use super::conflict_policy_registry::ConflictPolicyName;
use super::deletion_policy_registry::DeletionPolicyName;
use super::identity_matcher_registry::IdentityMatcherName;
use super::merge_base_registry::MergeBaseStrategyName;
use super::source_only_policy_registry::SourceOnlyPolicyName;
use super::strategy_registry::MergeStrategyName;
use super::BranchMergeStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchMergeRequestScopeFamily {
    FullBranch,
    SelectedNodes,
    SelectedAspects,
}

impl BranchMergeRequestScopeFamily {
    pub fn label(self) -> &'static str {
        match self {
            Self::FullBranch => "full-branch",
            Self::SelectedNodes => "selected-nodes",
            Self::SelectedAspects => "selected-aspects",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSelectedAspectRequestEntry {
    node: NodeId,
    aspect: Aspect,
}

impl SignalSelectedAspectRequestEntry {
    pub fn new(node: NodeId, aspect: Aspect) -> Self {
        Self { node, aspect }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn aspect(&self) -> Aspect {
        self.aspect
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BranchMergeRequestScope {
    #[default]
    FullBranch,
    SelectedNodes(Vec<NodeId>),
    SelectedAspects(Vec<SignalSelectedAspectRequestEntry>),
}

impl BranchMergeRequestScope {
    pub fn full_branch() -> Self {
        Self::FullBranch
    }

    pub fn selected_nodes(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self::SelectedNodes(nodes.into_iter().collect())
    }

    pub fn selected_aspects(
        aspects: impl IntoIterator<Item = SignalSelectedAspectRequestEntry>,
    ) -> Self {
        Self::SelectedAspects(aspects.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchMergeRequestDenial {
    EmptySelectedNodes,
    EmptySelectedAspects,
}

impl BranchMergeRequestDenial {
    pub fn into_signal_error(self) -> SignalError {
        match self {
            Self::EmptySelectedNodes => SignalError::invalid_input(
                "selected-node merge requests must name at least one source node",
            ),
            Self::EmptySelectedAspects => SignalError::invalid_input(
                "selected-aspect merge requests must name at least one aspect",
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedBranchMergeRequest {
    request: BranchMergeRequest,
    normalized_scope: NormalizedBranchMergeRequestScope,
}

impl NormalizedBranchMergeRequest {
    pub fn request(&self) -> &BranchMergeRequest {
        &self.request
    }

    pub fn normalized_scope(&self) -> &NormalizedBranchMergeRequestScope {
        &self.normalized_scope
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizedBranchMergeRequestScope {
    FullBranch {
        scope_digest: String,
    },
    SelectedNodes {
        selected_nodes: Vec<NodeId>,
        scope_digest: String,
    },
    SelectedAspects {
        selected_aspects: Vec<SignalSelectedAspectRequestEntry>,
        scope_digest: String,
    },
}

impl NormalizedBranchMergeRequestScope {
    pub fn family(&self) -> BranchMergeRequestScopeFamily {
        match self {
            Self::FullBranch { .. } => BranchMergeRequestScopeFamily::FullBranch,
            Self::SelectedNodes { .. } => BranchMergeRequestScopeFamily::SelectedNodes,
            Self::SelectedAspects { .. } => BranchMergeRequestScopeFamily::SelectedAspects,
        }
    }

    pub fn scope_digest(&self) -> &str {
        match self {
            Self::FullBranch { scope_digest }
            | Self::SelectedNodes { scope_digest, .. }
            | Self::SelectedAspects { scope_digest, .. } => scope_digest,
        }
    }

    pub fn selected_nodes(&self) -> &[NodeId] {
        match self {
            Self::SelectedNodes { selected_nodes, .. } => selected_nodes,
            _ => &[],
        }
    }

    pub fn selected_aspects(&self) -> &[SignalSelectedAspectRequestEntry] {
        match self {
            Self::SelectedAspects {
                selected_aspects, ..
            } => selected_aspects,
            _ => &[],
        }
    }

    pub fn is_full_branch(&self) -> bool {
        matches!(self, Self::FullBranch { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchMergeRequest {
    pub source_branch: SignalBranchHandle,
    pub target_branch: SignalBranchHandle,
    #[serde(default)]
    pub scope: BranchMergeRequestScope,
    #[serde(default)]
    pub strategy_name: Option<MergeStrategyName>,
    #[serde(default)]
    pub strategy_hint: Option<BranchMergeStrategy>,
    #[serde(default)]
    pub merge_base_name: Option<MergeBaseStrategyName>,
    #[serde(default)]
    pub conflict_policy_name: Option<ConflictPolicyName>,
    #[serde(default)]
    pub identity_matcher_name: Option<IdentityMatcherName>,
    #[serde(default)]
    pub source_only_policy_name: Option<SourceOnlyPolicyName>,
    #[serde(default)]
    pub deletion_policy_name: Option<DeletionPolicyName>,
    #[serde(default)]
    pub conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
    #[serde(default)]
    pub aspect_policy_bindings: Vec<AspectMergePolicyBinding>,
}

impl BranchMergeRequest {
    pub fn full_branch(
        source_branch: SignalBranchHandle,
        target_branch: SignalBranchHandle,
    ) -> Self {
        Self {
            source_branch,
            target_branch,
            scope: BranchMergeRequestScope::FullBranch,
            strategy_name: None,
            strategy_hint: None,
            merge_base_name: None,
            conflict_policy_name: None,
            identity_matcher_name: None,
            source_only_policy_name: None,
            deletion_policy_name: None,
            conflict_isolation_policy_name: None,
            aspect_policy_bindings: Vec::new(),
        }
    }

    pub fn selected_nodes(
        source_branch: SignalBranchHandle,
        target_branch: SignalBranchHandle,
        selected_nodes: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        Self {
            scope: BranchMergeRequestScope::selected_nodes(selected_nodes),
            ..Self::full_branch(source_branch, target_branch)
        }
    }

    pub fn selected_aspects(
        source_branch: SignalBranchHandle,
        target_branch: SignalBranchHandle,
        selected_aspects: impl IntoIterator<Item = SignalSelectedAspectRequestEntry>,
    ) -> Self {
        Self {
            scope: BranchMergeRequestScope::selected_aspects(selected_aspects),
            ..Self::full_branch(source_branch, target_branch)
        }
    }

    pub fn normalize_scope(
        &self,
    ) -> Result<NormalizedBranchMergeRequestScope, BranchMergeRequestDenial> {
        match &self.scope {
            BranchMergeRequestScope::FullBranch => {
                Ok(NormalizedBranchMergeRequestScope::FullBranch {
                    scope_digest: canonical_digest(&BranchMergeRequestScopeFamily::FullBranch),
                })
            }
            BranchMergeRequestScope::SelectedNodes(selected_nodes) => {
                let normalized = selected_nodes
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                if normalized.is_empty() {
                    return Err(BranchMergeRequestDenial::EmptySelectedNodes);
                }
                Ok(NormalizedBranchMergeRequestScope::SelectedNodes {
                    scope_digest: canonical_digest(&(
                        BranchMergeRequestScopeFamily::SelectedNodes,
                        &normalized,
                    )),
                    selected_nodes: normalized,
                })
            }
            BranchMergeRequestScope::SelectedAspects(selected_aspects) => {
                let mut normalized = selected_aspects.clone();
                normalized.sort_unstable_by_key(|entry| (entry.node(), entry.aspect().id()));
                normalized.dedup_by_key(|entry| (entry.node(), entry.aspect().id()));
                if normalized.is_empty() {
                    return Err(BranchMergeRequestDenial::EmptySelectedAspects);
                }
                Ok(NormalizedBranchMergeRequestScope::SelectedAspects {
                    scope_digest: canonical_digest(&(
                        BranchMergeRequestScopeFamily::SelectedAspects,
                        &normalized,
                    )),
                    selected_aspects: normalized,
                })
            }
        }
    }

    pub fn normalize(&self) -> Result<NormalizedBranchMergeRequest, BranchMergeRequestDenial> {
        Ok(NormalizedBranchMergeRequest {
            request: self.clone(),
            normalized_scope: self.normalize_scope()?,
        })
    }
}
