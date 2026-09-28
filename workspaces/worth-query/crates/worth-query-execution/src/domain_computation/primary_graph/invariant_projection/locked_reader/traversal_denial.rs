/// Why a relation traversal or mutation-target request in an invariant
/// projection was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantProjectionTraversalDenialKind {
    /// The relation is not installed.
    RelationNotInstalled,
    /// The operation does not declare this target as a decision read.
    UndeclaredDecisionTarget,
    /// The entity identity came from a different projection authority.
    ForeignIdentity,
    /// The reader is not for an admitted operation, so it cannot mint mutation
    /// targets.
    MutationTargetUnavailable,
    /// A relation endpoint could not be read.
    EndpointUnavailable,
    /// The relation does not declare exactly one related entity on the requested
    /// side.
    CardinalityContractMismatch,
    /// The source has no target, although the relation declares exactly one.
    MissingTarget,
    /// The source has more than one target, although the relation declares exactly
    /// one.
    MultipleTargets,
    /// The target has no source, although the relation declares exactly one.
    MissingSource,
    /// The target has more than one source, although the relation declares exactly
    /// one.
    MultipleSources,
    /// The traversal exceeded the projection's work budget.
    WorkBudgetExceeded,
}

/// Refusal of a relation traversal or mutation-target request in an invariant
/// projection.
///
/// [`Self::kind`] says why and [`Self::relation`] names the relation or
/// entity.
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

    pub(in crate::domain_computation::primary_graph) fn missing_source(
        relation: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::MissingSource,
            relation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn multiple_sources(
        relation: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryInvariantProjectionTraversalDenialKind::MultipleSources,
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
