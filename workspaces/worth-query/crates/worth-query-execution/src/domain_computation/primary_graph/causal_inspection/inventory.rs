#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CausalEvidenceOwner {
    Query,
    RuntimeBridge,
    Signal,
    Relational,
}

impl CausalEvidenceOwner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::RuntimeBridge => "runtime_bridge",
            Self::Signal => "signal",
            Self::Relational => "relational",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CausalEvidenceFamily {
    QueryInspection,
    QueryMutationCausality,
    QueryMutationProvenance,
    RelationalAuthority,
    RelationalDecision,
    BridgeRoute,
    BridgeEvaluation,
    BridgeSourceMaterialization,
    BridgeSourceFailure,
    BridgeContinuity,
    BridgeMerge,
    BridgeStructural,
    BridgeStream,
    BridgePreview,
    BridgeWriteback,
    BridgeMapper,
    BridgeReplay,
    SignalInvalidation,
    SignalEvaluation,
    SignalForensicAvailability,
    SignalReplayCursor,
    SignalLineage,
    SignalProvenance,
    Lineage,
    Provenance,
    Policy,
    Redaction,
}

impl CausalEvidenceFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryInspection => "query_inspection",
            Self::QueryMutationCausality => "query_mutation_causality",
            Self::QueryMutationProvenance => "query_mutation_provenance",
            Self::RelationalAuthority => "relational_authority",
            Self::RelationalDecision => "relational_decision",
            Self::BridgeRoute => "bridge_route",
            Self::BridgeEvaluation => "bridge_evaluation",
            Self::BridgeSourceMaterialization => "bridge_source_materialization",
            Self::BridgeSourceFailure => "bridge_source_failure",
            Self::BridgeContinuity => "bridge_continuity",
            Self::BridgeMerge => "bridge_merge",
            Self::BridgeStructural => "bridge_structural",
            Self::BridgeStream => "bridge_stream",
            Self::BridgePreview => "bridge_preview",
            Self::BridgeWriteback => "bridge_writeback",
            Self::BridgeMapper => "bridge_mapper",
            Self::BridgeReplay => "bridge_replay",
            Self::SignalInvalidation => "signal_invalidation",
            Self::SignalEvaluation => "signal_evaluation",
            Self::SignalForensicAvailability => "signal_forensic_availability",
            Self::SignalReplayCursor => "signal_replay_cursor",
            Self::SignalLineage => "signal_lineage",
            Self::SignalProvenance => "signal_provenance",
            Self::Lineage => "lineage",
            Self::Provenance => "provenance",
            Self::Policy => "policy",
            Self::Redaction => "redaction",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalEvidenceInventoryRow {
    owner: CausalEvidenceOwner,
    family: CausalEvidenceFamily,
    authority_surface: &'static str,
    query_reference_identity: &'static str,
}

impl CausalEvidenceInventoryRow {
    const fn new(
        owner: CausalEvidenceOwner,
        family: CausalEvidenceFamily,
        authority_surface: &'static str,
        query_reference_identity: &'static str,
    ) -> Self {
        Self {
            owner,
            family,
            authority_surface,
            query_reference_identity,
        }
    }

    pub fn owner(&self) -> CausalEvidenceOwner {
        self.owner
    }

    pub fn family(&self) -> CausalEvidenceFamily {
        self.family
    }

    pub fn authority_surface(&self) -> &'static str {
        self.authority_surface
    }

    pub fn query_reference_identity(&self) -> &'static str {
        self.query_reference_identity
    }
}

pub fn causal_evidence_inventory_rows() -> Vec<CausalEvidenceInventoryRow> {
    use CausalEvidenceFamily as Family;
    use CausalEvidenceOwner as Owner;

    vec![
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::QueryInspection,
            "WorthQueryInspection",
            "inspection_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::QueryMutationCausality,
            "WorthQueryMutationCausalityEvidence",
            "causality_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::QueryMutationProvenance,
            "WorthQueryMutationProvenanceEvidence",
            "feedback_provenance_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Relational,
            Family::RelationalAuthority,
            "commit/snapshot/branch authority identity",
            "relational_authority_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Relational,
            Family::RelationalDecision,
            "relational decision evidence",
            "relational_decision_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeRoute,
            "BridgeDiagnosticsFacade::route_records",
            "route_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeEvaluation,
            "BridgeDiagnosticsFacade::historical_evaluation_records",
            "evaluation_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeSourceMaterialization,
            "BridgeDiagnosticsFacade::source_materialization_records",
            "source_materialization_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeSourceFailure,
            "BridgeDiagnosticsFacade::source_failure_records",
            "source_failure_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeContinuity,
            "BridgeDiagnosticsFacade::continuity_records",
            "continuity_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeMerge,
            "BridgeDiagnosticsFacade::merge_records",
            "merge_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeStructural,
            "BridgeDiagnosticsFacade::structural_records",
            "structural_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeStream,
            "BridgeDiagnosticsFacade::stream_records",
            "stream_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgePreview,
            "BridgeDiagnosticsFacade::preview_records",
            "preview_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeWriteback,
            "BridgeDiagnosticsFacade::writeback_execution_records",
            "writeback_execution_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeMapper,
            "BridgeDiagnosticsFacade::mapper_records",
            "mapper_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::RuntimeBridge,
            Family::BridgeReplay,
            "BridgeDiagnosticsFacade::replay_records",
            "bridge_replay_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalInvalidation,
            "signal invalidation evidence",
            "signal_invalidation_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalEvaluation,
            "signal evaluation evidence",
            "signal_evaluation_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalForensicAvailability,
            "diagnostics_for_graph(...).forensic()",
            "forensic_availability_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalReplayCursor,
            "signal replay cursor evidence",
            "signal_replay_cursor_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalLineage,
            "signal lineage artifact evidence",
            "signal_lineage_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::SignalProvenance,
            "signal provenance artifact evidence",
            "signal_provenance_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Signal,
            Family::Lineage,
            "lineage artifact identity",
            "lineage_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::Provenance,
            "provenance artifact identity",
            "provenance_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::Policy,
            "query policy evidence",
            "policy_digest",
        ),
        CausalEvidenceInventoryRow::new(
            Owner::Query,
            Family::Redaction,
            "query redaction posture evidence",
            "redaction_digest",
        ),
    ]
}
