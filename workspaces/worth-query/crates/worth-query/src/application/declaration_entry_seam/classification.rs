#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationEntrySeamClassification {
    CanonicalReuse,
    QueryBoundaryAdapter,
    CompatibilityDebt,
    DeferredNeighbor,
    ForbiddenDuplicate,
}

impl WorthQueryDeclarationEntrySeamClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalReuse => "canonical_reuse",
            Self::QueryBoundaryAdapter => "query_boundary_adapter",
            Self::CompatibilityDebt => "compatibility_debt",
            Self::DeferredNeighbor => "deferred_neighbor",
            Self::ForbiddenDuplicate => "forbidden_duplicate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationEntryLowerOwnerCrate {
    Query,
    /// The truth authority a relational routing row lowers to. Every
    /// `RelationalTruthRouting` row names it, including grouped truth, whose
    /// artifact the Bridge now builds over Relational reads: the row records
    /// whose truth is routed, not which crate hosts the adapter.
    WorthRelational,
    WorthRuntimeBridge,
    WorthSignal,
}

impl WorthQueryDeclarationEntryLowerOwnerCrate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Query => "worth_query",
            Self::WorthRelational => "worth_relational",
            Self::WorthRuntimeBridge => "worth_runtime_bridge",
            Self::WorthSignal => "worth_signal",
        }
    }
}
