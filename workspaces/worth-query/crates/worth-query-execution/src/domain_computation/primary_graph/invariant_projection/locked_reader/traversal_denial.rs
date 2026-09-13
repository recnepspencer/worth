#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantProjectionTraversalDenialKind {
    RelationNotInstalled,
    UndeclaredDecisionTarget,
    ForeignIdentity,
    MutationTargetUnavailable,
    EndpointUnavailable,
    CardinalityContractMismatch,
    MissingTarget,
    MultipleTargets,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionTraversalDenial {
    kind: WorthQueryInvariantProjectionTraversalDenialKind,
    relation: String,
}

impl WorthQueryInvariantProjectionTraversalDenial {
    pub const fn kind(&self) -> WorthQueryInvariantProjectionTraversalDenialKind {
        self.kind
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryInvariantProjectionTraversalDenialKind,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            relation: relation.into(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn cardinality_contract_mismatch(
        relation: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::CardinalityContractMismatch,
            relation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn missing_target(
        relation: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::MissingTarget,
            relation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn multiple_targets(
        relation: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::MultipleTargets,
            relation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn foreign_identity(
        entity: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity,
            entity,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn mutation_target_unavailable(
        entity: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::MutationTargetUnavailable,
            entity,
        )
    }
}

impl std::fmt::Display for WorthQueryInvariantProjectionTraversalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invariant projection traversal denied: {:?} ({})",
            self.kind, self.relation
        )
    }
}

impl std::error::Error for WorthQueryInvariantProjectionTraversalDenial {}
