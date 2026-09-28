use crate::identity::hash_parts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BasisLifecycleReuseSurface {
    BridgeSubscriptionBasis,
    BridgeTruthViewBasis,
    BridgeContinuityBasis,
    BridgePreviewBasis,
    BridgeWritebackBasis,
    BridgeCausalEnvelopeBasis,
    RelationalTruthHistorySnapshotBasis,
    RelationalBridgeAdapterBasis,
    SignalSnapshotReplayLineageBasis,
}

impl BasisLifecycleReuseSurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BridgeSubscriptionBasis => "bridge_subscription_basis",
            Self::BridgeTruthViewBasis => "bridge_truth_view_basis",
            Self::BridgeContinuityBasis => "bridge_continuity_basis",
            Self::BridgePreviewBasis => "bridge_preview_basis",
            Self::BridgeWritebackBasis => "bridge_writeback_basis",
            Self::BridgeCausalEnvelopeBasis => "bridge_causal_envelope_basis",
            Self::RelationalTruthHistorySnapshotBasis => "relational_truth_history_snapshot_basis",
            Self::RelationalBridgeAdapterBasis => "relational_bridge_adapter_basis",
            Self::SignalSnapshotReplayLineageBasis => "signal_snapshot_replay_lineage_basis",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasisLifecycleReuseMatrixRow {
    surface: BasisLifecycleReuseSurface,
    owning_crate: &'static str,
    owning_facade: &'static str,
    authority_artifact: &'static str,
    query_wrapper: &'static str,
    allowed_carried_fields: &'static str,
    forbidden_duplicate_fields: &'static str,
    consuming_lanes: &'static str,
    posture: &'static str,
    enforcement_proof: &'static str,
    row_digest: String,
}

impl BasisLifecycleReuseMatrixRow {
    fn new(input: BasisLifecycleReuseMatrixRowInput) -> Self {
        let row_digest = hash_parts(&[
            format!("surface:{}", input.surface.as_str()),
            format!("crate:{}", input.owning_crate),
            format!("facade:{}", input.owning_facade),
            format!("authority:{}", input.authority_artifact),
            format!("wrapper:{}", input.query_wrapper),
            format!("allowed:{}", input.allowed_carried_fields),
            format!("forbidden:{}", input.forbidden_duplicate_fields),
            format!("lanes:{}", input.consuming_lanes),
            format!("posture:{}", input.posture),
            format!("proof:{}", input.enforcement_proof),
        ]);
        Self {
            surface: input.surface,
            owning_crate: input.owning_crate,
            owning_facade: input.owning_facade,
            authority_artifact: input.authority_artifact,
            query_wrapper: input.query_wrapper,
            allowed_carried_fields: input.allowed_carried_fields,
            forbidden_duplicate_fields: input.forbidden_duplicate_fields,
            consuming_lanes: input.consuming_lanes,
            posture: input.posture,
            enforcement_proof: input.enforcement_proof,
            row_digest,
        }
    }

    pub fn surface(&self) -> BasisLifecycleReuseSurface {
        self.surface
    }

    pub fn owning_crate(&self) -> &'static str {
        self.owning_crate
    }

    pub fn owning_facade(&self) -> &'static str {
        self.owning_facade
    }

    pub fn authority_artifact(&self) -> &'static str {
        self.authority_artifact
    }

    pub fn query_wrapper(&self) -> &'static str {
        self.query_wrapper
    }

    pub fn allowed_carried_fields(&self) -> &'static str {
        self.allowed_carried_fields
    }

    pub fn forbidden_duplicate_fields(&self) -> &'static str {
        self.forbidden_duplicate_fields
    }

    pub fn consuming_lanes(&self) -> &'static str {
        self.consuming_lanes
    }

    pub fn posture(&self) -> &'static str {
        self.posture
    }

    pub fn enforcement_proof(&self) -> &'static str {
        self.enforcement_proof
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

struct BasisLifecycleReuseMatrixRowInput {
    surface: BasisLifecycleReuseSurface,
    owning_crate: &'static str,
    owning_facade: &'static str,
    authority_artifact: &'static str,
    query_wrapper: &'static str,
    allowed_carried_fields: &'static str,
    forbidden_duplicate_fields: &'static str,
    consuming_lanes: &'static str,
    posture: &'static str,
    enforcement_proof: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasisLifecycleReuseMatrix {
    rows: Vec<BasisLifecycleReuseMatrixRow>,
    matrix_digest: String,
}

impl BasisLifecycleReuseMatrix {
    fn new(rows: Vec<BasisLifecycleReuseMatrixRow>) -> Self {
        let matrix_digest = hash_parts(
            &rows
                .iter()
                .map(|row| row.row_digest().to_string())
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            matrix_digest,
        }
    }

    pub fn rows(&self) -> &[BasisLifecycleReuseMatrixRow] {
        &self.rows
    }

    pub fn matrix_digest(&self) -> &str {
        &self.matrix_digest
    }

    pub fn row_for(
        &self,
        surface: BasisLifecycleReuseSurface,
    ) -> Option<&BasisLifecycleReuseMatrixRow> {
        self.rows.iter().find(|row| row.surface() == surface)
    }
}

pub fn basis_lifecycle_reuse_matrix() -> BasisLifecycleReuseMatrix {
    use BasisLifecycleReuseSurface::*;
    BasisLifecycleReuseMatrix::new(vec![
        row(
            BridgeSubscriptionBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::RuntimeBridge::admit_subscription",
            "ValidatedSubscriptionBasisBinding",
            "LowerRuntimeBasisEvidence",
            "identity/digest/receipt/denial/counter labels",
            "ValidatedSubscriptionBasisBinding fields",
            "subscription_declaration,subscription_activation",
            "reused",
            "subscription_basis_binding_request_constructor_private",
        ),
        row(
            BridgeTruthViewBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::TruthSnapshotReader",
            "BridgeTruthViewAuthorityBasis",
            "LowerRuntimeBasisEvidence",
            "authority label and facade-returned digest",
            "BridgeTruthViewAuthorityBasis fields",
            "observation,materialization",
            "reused",
            "basis_lifecycle_lower_runtime_evidence_constructor_private",
        ),
        row(
            BridgeContinuityBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::RuntimeBridge::deliver_continuity",
            "BridgeContinuityAuthorityBasis",
            "LowerRuntimeBasisEvidence",
            "continuity digest and denial class",
            "BridgeContinuityAuthorityBasis fields",
            "replay,inspection",
            "reused",
            "basis_lifecycle_lower_runtime_bound_basis_constructor_private",
        ),
        row(
            BridgePreviewBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::RuntimeBridge::admit_subscription_preview_basis",
            "BridgeSubscriptionPreviewBasisBinding",
            "LowerRuntimeBasisEvidence",
            "preview basis digest and receipt digest",
            "BridgeSubscriptionPreviewBasisBinding fields",
            "inspection,preview_closeout",
            "reused",
            "runtime_preview_basis_admission_constructor_private",
        ),
        row(
            BridgeWritebackBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade bridge writeback contracts",
            "BridgeWritebackCausalityBasis",
            "LowerRuntimeBasisEvidence",
            "writeback receipt digest and authority label",
            "BridgeWriteback*Basis fields",
            "mutation_preparation",
            "reused",
            "runtime_write_receipt_inspection_constructor_private",
        ),
        row(
            BridgeCausalEnvelopeBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope",
            "BridgeCausalEvidenceBinding",
            "LowerRuntimeBasisEvidence",
            "causal envelope/evidence digests",
            "BridgeCausalEvidenceBinding fields",
            "inspection",
            "reused",
            "facade_does_not_export_bridge_causal_constructors",
        ),
        row(
            RelationalTruthHistorySnapshotBasis,
            "worth-relational",
            "worth_relational::facade",
            "BranchHead/SnapshotHandle/CanonicalCommitEnvelope",
            "LowerRuntimeBasisEvidence",
            "relational authority digest and identity label",
            "BranchHead/SnapshotHandle/CanonicalCommitEnvelope fields",
            "observation,mutation_preparation,replay,materialization",
            "reused",
            "private_resolved_snapshot_basis_fields",
        ),
        row(
            RelationalBridgeAdapterBasis,
            "worth-runtime-bridge",
            "worth_runtime_bridge::facade::RuntimeBridgeRelationalSource",
            "RuntimeBridgeRelationalSource",
            "LowerRuntimeBasisEvidence",
            "adapter identity and bridge-facing digest",
            "commit/snapshot loader internals",
            "bridge_bound_observation,replay",
            "reused",
            "facade_does_not_export_bridge_causal_constructors",
        ),
        row(
            SignalSnapshotReplayLineageBasis,
            "worth-signal",
            "worth_signal::facade",
            "SignalSnapshotV1/ReplayCursor/LineageRecord",
            "LowerRuntimeBasisEvidence",
            "signal evidence digest and missing-evidence denial",
            "SignalSnapshotV1/ReplayCursor/LineageRecord fields",
            "observation,inspection",
            "reused",
            "basis_lifecycle_lower_runtime_evidence_constructor_private",
        ),
    ])
}

pub fn basis_lifecycle_reuse_matrix_digest() -> String {
    basis_lifecycle_reuse_matrix().matrix_digest().to_string()
}

pub fn basis_lifecycle_signal_authority_digest() -> String {
    basis_lifecycle_reuse_matrix()
        .row_for(BasisLifecycleReuseSurface::SignalSnapshotReplayLineageBasis)
        .expect("signal authority reuse row must be present")
        .row_digest()
        .to_string()
}

pub fn basis_lifecycle_adapter_shape_contract_digest() -> String {
    hash_parts(&[
        "adapter_shape_contract_v1".to_string(),
        "allowed:owning_crate,authority_family,facade_type,identity_digest,receipt_digest,denial_class,counters,support_posture,authority_label,query_lifecycle_digest".to_string(),
        "forbidden:reconstructive_authority_fields,private_module_fields,fresh_lower_runtime_constructors,replay_restore_reresolve_material".to_string(),
    ])
}

fn row(
    surface: BasisLifecycleReuseSurface,
    owning_crate: &'static str,
    owning_facade: &'static str,
    authority_artifact: &'static str,
    query_wrapper: &'static str,
    allowed_carried_fields: &'static str,
    forbidden_duplicate_fields: &'static str,
    consuming_lanes: &'static str,
    posture: &'static str,
    enforcement_proof: &'static str,
) -> BasisLifecycleReuseMatrixRow {
    BasisLifecycleReuseMatrixRow::new(BasisLifecycleReuseMatrixRowInput {
        surface,
        owning_crate,
        owning_facade,
        authority_artifact,
        query_wrapper,
        allowed_carried_fields,
        forbidden_duplicate_fields,
        consuming_lanes,
        posture,
        enforcement_proof,
    })
}

#[cfg(test)]
mod tests;
