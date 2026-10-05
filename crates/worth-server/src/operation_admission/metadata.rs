#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerProductSessionCoordinationTarget {
    SessionCreation,
    ExistingSession { product_session_identity: String },
}

impl WorthServerProductSessionCoordinationTarget {
    fn canonical_label(&self) -> &str {
        match self {
            Self::SessionCreation => "session-creation",
            Self::ExistingSession {
                product_session_identity,
            } => product_session_identity,
        }
    }

    pub fn product_session_identity(&self) -> Option<&str> {
        match self {
            Self::SessionCreation => None,
            Self::ExistingSession {
                product_session_identity,
            } => Some(product_session_identity),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerOperationAuthorityMetadata {
    SharedReadOnly {
        basis_kind: String,
        basis_digest: String,
        authority_label: String,
        product_support_posture: String,
    },
    DeterministicSubmission {
        submission_lane: String,
        journal_posture: String,
        base_digest_posture: String,
        idempotency_posture: String,
    },
    ProductDraftMutation {
        product_session_identity: String,
        draft_scope: String,
        base_digest_posture: String,
        idempotency_posture: String,
    },
    DurableProductMutation {
        authority_scope: String,
        expected_basis_digest: String,
        idempotency_key: String,
        durability_contract_digest: String,
    },
    ProductSessionCoordination {
        target: WorthServerProductSessionCoordinationTarget,
        coordination_lane: String,
    },
    BinaryStreaming {
        stream_kind: String,
        preflight_posture: String,
        size_posture: String,
        cancellation_posture: String,
        partial_failure_posture: String,
    },
    DiagnosticsOnly {
        diagnostics_lane: String,
    },
    LeaseCoordination {
        lease_target: String,
        resume_basis_digest: Option<String>,
        coordination_lane: String,
    },
}

impl WorthServerOperationAuthorityMetadata {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        match self {
            Self::SharedReadOnly {
                basis_kind,
                basis_digest,
                authority_label,
                product_support_posture,
            } => string(basis_kind)
                .saturating_add(string(basis_digest))
                .saturating_add(string(authority_label))
                .saturating_add(string(product_support_posture)),
            _ => u64::MAX,
        }
    }

    pub fn shared_read(
        basis_kind: impl Into<String>,
        basis_digest: impl Into<String>,
        authority_label: impl Into<String>,
    ) -> Self {
        Self::shared_read_with_support_posture(
            basis_kind,
            basis_digest,
            authority_label,
            "production-admitted",
        )
    }

    pub fn shared_read_with_support_posture(
        basis_kind: impl Into<String>,
        basis_digest: impl Into<String>,
        authority_label: impl Into<String>,
        product_support_posture: impl Into<String>,
    ) -> Self {
        Self::SharedReadOnly {
            basis_kind: basis_kind.into(),
            basis_digest: basis_digest.into(),
            authority_label: authority_label.into(),
            product_support_posture: product_support_posture.into(),
        }
    }

    pub fn deterministic_submission(
        submission_lane: impl Into<String>,
        journal_posture: impl Into<String>,
        base_digest_posture: impl Into<String>,
        idempotency_posture: impl Into<String>,
    ) -> Self {
        Self::DeterministicSubmission {
            submission_lane: submission_lane.into(),
            journal_posture: journal_posture.into(),
            base_digest_posture: base_digest_posture.into(),
            idempotency_posture: idempotency_posture.into(),
        }
    }

    pub fn binary_streaming(
        stream_kind: impl Into<String>,
        preflight_posture: impl Into<String>,
        size_posture: impl Into<String>,
        cancellation_posture: impl Into<String>,
        partial_failure_posture: impl Into<String>,
    ) -> Self {
        Self::BinaryStreaming {
            stream_kind: stream_kind.into(),
            preflight_posture: preflight_posture.into(),
            size_posture: size_posture.into(),
            cancellation_posture: cancellation_posture.into(),
            partial_failure_posture: partial_failure_posture.into(),
        }
    }

    pub fn product_draft_mutation(
        product_session_identity: impl Into<String>,
        draft_scope: impl Into<String>,
        base_digest_posture: impl Into<String>,
        idempotency_posture: impl Into<String>,
    ) -> Self {
        Self::ProductDraftMutation {
            product_session_identity: product_session_identity.into(),
            draft_scope: draft_scope.into(),
            base_digest_posture: base_digest_posture.into(),
            idempotency_posture: idempotency_posture.into(),
        }
    }

    pub fn durable_product_mutation(
        authority_scope: impl Into<String>,
        expected_basis_digest: impl Into<String>,
        idempotency_key: impl Into<String>,
        durability_contract_digest: impl Into<String>,
    ) -> Self {
        Self::DurableProductMutation {
            authority_scope: authority_scope.into(),
            expected_basis_digest: expected_basis_digest.into(),
            idempotency_key: idempotency_key.into(),
            durability_contract_digest: durability_contract_digest.into(),
        }
    }

    pub fn product_session_coordination(
        target: WorthServerProductSessionCoordinationTarget,
        coordination_lane: impl Into<String>,
    ) -> Self {
        Self::ProductSessionCoordination {
            target,
            coordination_lane: coordination_lane.into(),
        }
    }

    pub fn lease_coordination(
        lease_target: impl Into<String>,
        resume_basis_digest: Option<String>,
        coordination_lane: impl Into<String>,
    ) -> Self {
        Self::LeaseCoordination {
            lease_target: lease_target.into(),
            resume_basis_digest,
            coordination_lane: coordination_lane.into(),
        }
    }

    pub(crate) fn canonical_digest(&self) -> String {
        match self {
            Self::SharedReadOnly {
                basis_kind,
                basis_digest,
                authority_label,
                product_support_posture,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=shared-read-only|basis_kind={basis_kind}|basis_digest={basis_digest}|label={authority_label}|product_support_posture={product_support_posture}"
            ),
            Self::DeterministicSubmission {
                submission_lane,
                journal_posture,
                base_digest_posture,
                idempotency_posture,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=deterministic-submission|lane={submission_lane}|journal={journal_posture}|base={base_digest_posture}|idempotency={idempotency_posture}"
            ),
            Self::ProductDraftMutation {
                product_session_identity,
                draft_scope,
                base_digest_posture,
                idempotency_posture,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=product-draft-mutation|session={product_session_identity}|draft={draft_scope}|base={base_digest_posture}|idempotency={idempotency_posture}"
            ),
            Self::DurableProductMutation {
                authority_scope,
                expected_basis_digest,
                idempotency_key,
                durability_contract_digest,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=durable-product-mutation|scope={authority_scope}|basis={expected_basis_digest}|idempotency={idempotency_key}|contract={durability_contract_digest}"
            ),
            Self::ProductSessionCoordination {
                target,
                coordination_lane,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=product-session-coordination|target={}|lane={coordination_lane}",
                target.canonical_label(),
            ),
            Self::BinaryStreaming {
                stream_kind,
                preflight_posture,
                size_posture,
                cancellation_posture,
                partial_failure_posture,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=binary-streaming|stream={stream_kind}|preflight={preflight_posture}|size={size_posture}|cancellation={cancellation_posture}|partial_failure={partial_failure_posture}"
            ),
            Self::DiagnosticsOnly { diagnostics_lane } => format!(
                "worth-server-operation-authority-metadata-v1|kind=diagnostics-only|lane={diagnostics_lane}"
            ),
            Self::LeaseCoordination {
                lease_target,
                resume_basis_digest,
                coordination_lane,
            } => format!(
                "worth-server-operation-authority-metadata-v1|kind=lease-coordination|target={lease_target}|basis={}|lane={coordination_lane}",
                resume_basis_digest.as_deref().unwrap_or("none")
            ),
        }
    }

    pub fn submission_lane(&self) -> Option<&str> {
        match self {
            Self::DeterministicSubmission {
                submission_lane, ..
            } => Some(submission_lane),
            _ => None,
        }
    }

    pub fn product_draft_scope(&self) -> Option<(&str, &str)> {
        match self {
            Self::ProductDraftMutation {
                product_session_identity,
                draft_scope,
                ..
            } => Some((product_session_identity, draft_scope)),
            _ => None,
        }
    }

    pub fn durable_product_mutation_scope(&self) -> Option<(&str, &str, &str)> {
        match self {
            Self::DurableProductMutation {
                authority_scope,
                expected_basis_digest,
                idempotency_key,
                ..
            } => Some((authority_scope, expected_basis_digest, idempotency_key)),
            _ => None,
        }
    }

    pub fn product_session_coordination_target(
        &self,
    ) -> Option<(&WorthServerProductSessionCoordinationTarget, &str)> {
        match self {
            Self::ProductSessionCoordination {
                target,
                coordination_lane,
            } => Some((target, coordination_lane)),
            _ => None,
        }
    }
}
