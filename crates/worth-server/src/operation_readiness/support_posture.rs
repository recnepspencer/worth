use crate::{WorthServerOperationAuthorityMetadata, WorthServerQuerySupportPosture};

use super::WorthServerOperationSupportCompositionReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationSupportPosture {
    query_support_posture: Option<WorthServerQuerySupportPosture>,
    product_support_label: String,
    shared_read_comparable: bool,
    composition_receipt: WorthServerOperationSupportCompositionReceipt,
    canonical_digest: String,
}

impl WorthServerOperationSupportPosture {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        self.query_support_posture
            .as_ref()
            .map_or(0, |posture| posture.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.product_support_label))
            .saturating_add(self.composition_receipt.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.canonical_digest))
    }
    pub(crate) fn new(
        query_support_posture: Option<WorthServerQuerySupportPosture>,
        authority_metadata: &WorthServerOperationAuthorityMetadata,
        composition_receipt: WorthServerOperationSupportCompositionReceipt,
    ) -> Self {
        let product_support_label = product_support_label(authority_metadata);
        let shared_read_comparable = shared_read_comparable(authority_metadata);
        let canonical_digest = format!(
            "worth-server-operation-support-posture-v1|query={}|product={product_support_label}|shared_read_comparable={shared_read_comparable}|receipt={}",
            query_support_posture
                .as_ref()
                .map(WorthServerQuerySupportPosture::canonical_label)
                .unwrap_or_else(|| "none".to_string()),
            composition_receipt.canonical_digest(),
        );
        Self {
            query_support_posture,
            product_support_label,
            shared_read_comparable,
            composition_receipt,
            canonical_digest,
        }
    }

    pub fn query_support_posture(&self) -> Option<&WorthServerQuerySupportPosture> {
        self.query_support_posture.as_ref()
    }

    pub fn product_support_label(&self) -> &str {
        &self.product_support_label
    }

    pub fn shared_read_comparable(&self) -> bool {
        self.shared_read_comparable
    }

    pub fn composition_receipt(&self) -> &WorthServerOperationSupportCompositionReceipt {
        &self.composition_receipt
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

fn product_support_label(authority_metadata: &WorthServerOperationAuthorityMetadata) -> String {
    match authority_metadata {
        WorthServerOperationAuthorityMetadata::SharedReadOnly {
            basis_kind,
            product_support_posture,
            ..
        } => {
            format!("shared-read:{basis_kind}:{product_support_posture}")
        }
        WorthServerOperationAuthorityMetadata::DeterministicSubmission {
            submission_lane, ..
        } => format!("deterministic-submission:{submission_lane}"),
        WorthServerOperationAuthorityMetadata::ProductDraftMutation { draft_scope, .. } => {
            format!("product-draft-mutation:{draft_scope}")
        }
        WorthServerOperationAuthorityMetadata::DurableProductMutation {
            authority_scope, ..
        } => format!("durable-product-mutation:{authority_scope}"),
        WorthServerOperationAuthorityMetadata::ProductSessionCoordination {
            coordination_lane,
            ..
        } => format!("product-session-coordination:{coordination_lane}"),
        WorthServerOperationAuthorityMetadata::BinaryStreaming { stream_kind, .. } => {
            format!("binary-streaming:{stream_kind}")
        }
        WorthServerOperationAuthorityMetadata::DiagnosticsOnly { diagnostics_lane } => {
            format!("diagnostics-only:{diagnostics_lane}")
        }
        WorthServerOperationAuthorityMetadata::LeaseCoordination {
            coordination_lane, ..
        } => format!("lease-coordination:{coordination_lane}"),
    }
}

fn shared_read_comparable(authority_metadata: &WorthServerOperationAuthorityMetadata) -> bool {
    matches!(
        authority_metadata,
        WorthServerOperationAuthorityMetadata::SharedReadOnly {
            basis_kind,
            product_support_posture,
            ..
        } if product_support_posture == "production-admitted"
            && (basis_kind == "query-shared-read-basis"
                || basis_kind == "query-derived"
                || basis_kind == "primary-graph-application"
                || basis_kind == "product-session-derived"
                || basis_kind == "durable-product-derived")
    )
}
