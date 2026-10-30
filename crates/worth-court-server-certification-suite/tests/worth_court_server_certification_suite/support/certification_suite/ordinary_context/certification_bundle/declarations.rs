#![allow(dead_code)]

pub(crate) use std::collections::BTreeMap;

pub(crate) use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
pub(crate) use {
    worth_server::WorthServerDenialBoundary, worth_server::WorthServerOperatorEvidenceRecord,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum WorthServerCertificationOutputDigest {
    SurfaceContract,
    Declaration,
    DeclarationSupport,
    Handoff,
    SupportPosture,
    Branch,
    Workspace,
    SupportMatrix,
    ViewShape,
    RetainedState,
    Basis,
    Remask,
    AsyncResult,
    TemporalState,
    Policy,
    FactReceipt,
    Materialization,
    CounterSnapshot,
    DeliveryRequest,
    ResumeMode,
    FreshnessMode,
    DeliveryClass,
    DenialCode,
    DenialDetail,
}

impl WorthServerCertificationOutputDigest {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::SurfaceContract => "surface_contract_digest",
            Self::Declaration => "declaration_digest",
            Self::DeclarationSupport => "declaration_support_digest",
            Self::Handoff => "handoff_digest",
            Self::SupportPosture => "support_posture_digest",
            Self::Branch => "branch_digest",
            Self::Workspace => "workspace_digest",
            Self::SupportMatrix => "support_matrix_digest",
            Self::ViewShape => "view_shape",
            Self::RetainedState => "retained_state_digest",
            Self::Basis => "basis_digest",
            Self::Remask => "remask_digest",
            Self::AsyncResult => "async_result_digest",
            Self::TemporalState => "temporal_state_digest",
            Self::Policy => "policy_digest",
            Self::FactReceipt => "fact_receipt_digest",
            Self::Materialization => "materialization_digest",
            Self::CounterSnapshot => "counter_snapshot_digest",
            Self::DeliveryRequest => "delivery_request_digest",
            Self::ResumeMode => "resume_mode",
            Self::FreshnessMode => "freshness_mode",
            Self::DeliveryClass => "delivery_class",
            Self::DenialCode => "denial_code",
            Self::DenialDetail => "denial_detail",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthServerCertificationField {
    RequestContextDigest,
    ResponseDigest,
    ProvenanceDigest,
    FailureDigest,
    CounterSnapshot,
    Output(WorthServerCertificationOutputDigest),
}

impl WorthServerCertificationField {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RequestContextDigest => "request_context_digest",
            Self::ResponseDigest => "response_digest",
            Self::ProvenanceDigest => "provenance_digest",
            Self::FailureDigest => "failure_digest",
            Self::CounterSnapshot => "counter_snapshot",
            Self::Output(output) => output.label(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthServerCertificationBundle {
    pub(crate) request_context_digest: String,
    pub(crate) response_digest: String,
    pub(crate) provenance_digest: String,
    pub(crate) failure_digest: Option<String>,
    pub(crate) output_digests: BTreeMap<WorthServerCertificationOutputDigest, String>,
    pub(crate) counter_snapshot: BTreeMap<String, u64>,
    pub(crate) support_truth_kind: FoundationalBoundaryEvidenceSupportTruthKind,
    pub(crate) support_attachment_present: bool,
}

impl WorthServerCertificationBundle {
    pub(crate) fn from_response_and_evidence(
        request_context_digest: String,
        response: worth_server::WorthServerResponseEnvelope,
        evidence: WorthServerOperatorEvidenceRecord,
    ) -> Self {
        Self {
            request_context_digest,
            response_digest: response.canonical_digest().to_string(),
            provenance_digest: provenance_digest(response.provenance()),
            failure_digest: response.denial().map(failure_digest),
            output_digests: std::collections::BTreeMap::new(),
            counter_snapshot: counter_snapshot(&evidence),
            support_truth_kind: evidence.support_truth_kind(),
            support_attachment_present: evidence
                .materialized_attachment_bundle()
                .support()
                .is_some(),
        }
    }

    pub(crate) fn request_context_digest(&self) -> &str {
        &self.request_context_digest
    }

    pub(crate) fn response_digest(&self) -> &str {
        &self.response_digest
    }

    pub(crate) fn provenance_digest(&self) -> &str {
        &self.provenance_digest
    }

    pub(crate) fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    pub(crate) fn support_truth_kind(&self) -> FoundationalBoundaryEvidenceSupportTruthKind {
        self.support_truth_kind
    }

    pub(crate) fn counter_snapshot(&self) -> &BTreeMap<String, u64> {
        &self.counter_snapshot
    }

    pub(crate) fn output_digest(&self, name: WorthServerCertificationOutputDigest) -> Option<&str> {
        self.output_digests.get(&name).map(String::as_str)
    }

    pub(crate) fn with_output_digest(
        mut self,
        name: WorthServerCertificationOutputDigest,
        digest: impl Into<String>,
    ) -> Self {
        self.output_digests.insert(name, digest.into());
        self
    }

    pub(crate) fn with_optional_output_digest(
        mut self,
        name: WorthServerCertificationOutputDigest,
        digest: Option<impl Into<String>>,
    ) -> Self {
        if let Some(digest) = digest {
            self.output_digests.insert(name, digest.into());
        }
        self
    }

    pub(crate) fn support_attachment_present(&self) -> bool {
        self.support_attachment_present
    }

    pub(crate) fn counter_value(&self, name: &str) -> Option<u64> {
        self.counter_snapshot.get(name).copied()
    }
}

pub(crate) fn counter_snapshot(
    evidence: &WorthServerOperatorEvidenceRecord,
) -> BTreeMap<String, u64> {
    evidence
        .counter_receipt()
        .receipt()
        .counter_rows()
        .iter()
        .map(|row| (row.name().as_str().to_string(), row.observed_count()))
        .collect()
}

pub(crate) fn provenance_digest(
    provenance: &worth_foundational::facade::FoundationalBoundaryEvidenceProvenanceArtifact,
) -> String {
    format!(
        "locality={:?};freshness={:?};source={:?};authority={};strategy={};profile={};comparison={};canonical={};support_contexts={}",
        provenance.locality(),
        provenance.freshness_posture(),
        provenance.source_basis().kind(),
        provenance.authority_path().is_some(),
        provenance.strategy_basis().is_some(),
        provenance.profile_basis().is_some(),
        provenance.comparison_basis().is_some(),
        provenance.canonical_digest_basis().is_some(),
        provenance.support_context_attachments().len(),
    )
}

pub(crate) fn failure_digest(denial: &worth_server::WorthServerDenialEnvelope) -> String {
    match denial.cause().boundary() {
        worth_server::WorthServerDenialBoundary::RequestContext => format!(
            "request_context:{:?}:{}",
            denial
                .request_context_code()
                .expect("request-context denial code"),
            denial.cause().detail()
        ),
        worth_server::WorthServerDenialBoundary::Middleware => format!(
            "middleware:{:?}:{:?}:{:?}:{}",
            denial.middleware_code().expect("middleware denial code"),
            denial
                .middleware_priority()
                .expect("middleware denial priority"),
            denial.middleware_step().expect("middleware denial step"),
            denial.cause().detail()
        ),
        worth_server::WorthServerDenialBoundary::QueryHandoff => format!(
            "query_handoff:{:?}:{}",
            denial
                .query_handoff_code()
                .expect("query handoff denial code"),
            denial.cause().detail()
        ),
    }
}
