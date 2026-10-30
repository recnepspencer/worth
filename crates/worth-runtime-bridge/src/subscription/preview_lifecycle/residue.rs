use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::subscription::{
    BridgePreviewActiveSubscription, BridgePreviewActiveSubscriptionIdentity,
    BridgeSubscriptionCounters, BridgeSubscriptionPreviewBasisIdentity,
    BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity,
    BridgeSubscriptionPreviewResidueScopeIdentity, BridgeSubscriptionPreviewScopeIdentity,
    BridgeSubscriptionPreviewWorkKind, BridgeSubscriptionPreviewWorkTrace,
    BridgeSubscriptionPreviewWorkTraceIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeSubscriptionPreviewLifecycleResidueKind {
    TemporalWake,
    InflightAsync,
    CompletionWriteback,
    SharedDelivery,
}

impl BridgeSubscriptionPreviewLifecycleResidueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalWake => "temporal_wake",
            Self::InflightAsync => "inflight_async",
            Self::CompletionWriteback => "completion_writeback",
            Self::SharedDelivery => "shared_delivery",
        }
    }

    pub const fn all() -> [Self; 4] {
        [
            Self::TemporalWake,
            Self::InflightAsync,
            Self::CompletionWriteback,
            Self::SharedDelivery,
        ]
    }

    pub(crate) fn preview_work_kind(self) -> BridgeSubscriptionPreviewWorkKind {
        match self {
            Self::TemporalWake => BridgeSubscriptionPreviewWorkKind::Routing,
            Self::InflightAsync => BridgeSubscriptionPreviewWorkKind::Continuation,
            Self::CompletionWriteback => BridgeSubscriptionPreviewWorkKind::Diagnostics,
            Self::SharedDelivery => BridgeSubscriptionPreviewWorkKind::Delivery,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleResidueInput {
    kind: BridgeSubscriptionPreviewLifecycleResidueKind,
    residue_count: usize,
    evidence_digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleResidueInput {
    pub fn from_preview_work_trace(
        kind: BridgeSubscriptionPreviewLifecycleResidueKind,
        residue_count: usize,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
    ) -> Self {
        let record_digest = preview_work_trace.record_digest_for(kind.preview_work_kind());
        Self {
            kind,
            residue_count,
            evidence_digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-residue-evidence|trace={}|scope={}|record={record_digest}|kind={}",
                preview_work_trace.digest(),
                preview_work_trace.preview_residue_scope_identity().as_str(),
                kind.as_str(),
            )),
        }
    }

    pub fn custom(
        kind: BridgeSubscriptionPreviewLifecycleResidueKind,
        residue_count: usize,
        evidence_digest: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            residue_count,
            evidence_digest: evidence_digest.into(),
        }
    }

    pub fn kind(&self) -> BridgeSubscriptionPreviewLifecycleResidueKind {
        self.kind
    }

    pub fn residue_count(&self) -> usize {
        self.residue_count
    }

    pub fn evidence_digest(&self) -> &str {
        self.evidence_digest.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejectionKind {
    PreviewWorkTraceMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejection {
    rejection_kind: BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejectionKind,
    rejection_context: Arc<str>,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejection {
    fn preview_work_trace_mismatch(
        preview_active: &BridgePreviewActiveSubscription,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
    ) -> Self {
        let rejection_context = Arc::<str>::from(format!(
            "preview-active={}|trace-preview-active={}|preview-scope={}|trace-scope={}",
            preview_active
                .preview_active_subscription_identity()
                .as_str(),
            preview_work_trace
                .preview_active_subscription_identity()
                .as_str(),
            preview_active.preview_scope_identity().as_str(),
            preview_work_trace.preview_scope_identity().as_str(),
        ));
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-residue-envelope-rejection|kind=preview_work_trace_mismatch|context={rejection_context}"
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind:
                BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejectionKind::PreviewWorkTraceMismatch,
            rejection_context,
            counters: BridgeSubscriptionCounters::from_subscription_preview_lifecycle_residue_envelope(),
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-residue-envelope-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejectionKind {
        self.rejection_kind
    }

    pub fn rejection_context(&self) -> &str {
        self.rejection_context.as_ref()
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleResidueRecord {
    kind: BridgeSubscriptionPreviewLifecycleResidueKind,
    residue_count: usize,
    evidence_digest: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleResidueRecord {
    fn from_input(
        preview_residue_scope_identity: &BridgeSubscriptionPreviewResidueScopeIdentity,
        input: BridgeSubscriptionPreviewLifecycleResidueInput,
    ) -> Self {
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-residue-record|scope={}|kind={}|residue-count={}|evidence={}",
            preview_residue_scope_identity.as_str(),
            input.kind.as_str(),
            input.residue_count,
            input.evidence_digest.as_ref(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            kind: input.kind,
            residue_count: input.residue_count,
            evidence_digest: input.evidence_digest,
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-residue-record:sha256:{digest:x}"
            )),
        }
    }

    pub fn kind(&self) -> BridgeSubscriptionPreviewLifecycleResidueKind {
        self.kind
    }

    pub fn residue_count(&self) -> usize {
        self.residue_count
    }

    pub fn evidence_digest(&self) -> &str {
        self.evidence_digest.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleResidueEnvelope {
    residue_envelope_identity: BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity,
    preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
    preview_basis_identity: BridgeSubscriptionPreviewBasisIdentity,
    preview_scope_identity: BridgeSubscriptionPreviewScopeIdentity,
    preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    preview_work_trace_identity: BridgeSubscriptionPreviewWorkTraceIdentity,
    preview_work_trace_digest: Arc<str>,
    residue_records: Arc<[BridgeSubscriptionPreviewLifecycleResidueRecord]>,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleResidueEnvelope {
    pub(crate) fn capture(
        preview_active: &BridgePreviewActiveSubscription,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
        residue_inputs: Vec<BridgeSubscriptionPreviewLifecycleResidueInput>,
    ) -> Result<Self, BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejection> {
        if preview_work_trace.preview_active_subscription_identity()
            != preview_active.preview_active_subscription_identity()
            || preview_work_trace.preview_scope_identity()
                != preview_active.preview_scope_identity()
        {
            return Err(
                BridgeSubscriptionPreviewLifecycleResidueEnvelopeRejection::preview_work_trace_mismatch(
                    preview_active,
                    preview_work_trace,
                ),
            );
        }

        let mut residue_records = residue_inputs
            .into_iter()
            .map(|input| {
                BridgeSubscriptionPreviewLifecycleResidueRecord::from_input(
                    preview_active.preview_residue_scope_identity(),
                    input,
                )
            })
            .collect::<Vec<_>>();
        residue_records.sort_by(|left, right| {
            left.kind()
                .cmp(&right.kind())
                .then_with(|| left.digest().cmp(right.digest()))
        });
        let residue_digests = residue_records
            .iter()
            .map(BridgeSubscriptionPreviewLifecycleResidueRecord::digest)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-residue-envelope|preview-active={}|preview-basis={}|preview-scope={}|residue-scope={}|preview-work-trace={}|residue-records={}",
            preview_active.preview_active_subscription_identity().as_str(),
            preview_active.preview_basis_identity().as_str(),
            preview_active.preview_scope_identity().as_str(),
            preview_active.preview_residue_scope_identity().as_str(),
            preview_work_trace.preview_work_trace_identity().as_str(),
            residue_digests,
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            residue_envelope_identity:
                BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity::admit_bridge_owned(
                    format!(
                    "bridge-subscription-preview-lifecycle-residue-envelope-id:sha256:{digest:x}"
                ),
                ),
            preview_active_subscription_identity: preview_active
                .preview_active_subscription_identity()
                .clone(),
            preview_basis_identity: preview_active.preview_basis_identity().clone(),
            preview_scope_identity: preview_active.preview_scope_identity().clone(),
            preview_residue_scope_identity: preview_active.preview_residue_scope_identity().clone(),
            preview_work_trace_identity: preview_work_trace.preview_work_trace_identity().clone(),
            preview_work_trace_digest: Arc::from(preview_work_trace.digest()),
            residue_records: Arc::from(residue_records),
            counters:
                BridgeSubscriptionCounters::from_subscription_preview_lifecycle_residue_envelope(),
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-residue-envelope:sha256:{digest:x}"
            )),
        })
    }

    pub fn residue_envelope_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity {
        &self.residue_envelope_identity
    }

    pub fn preview_active_subscription_identity(&self) -> &BridgePreviewActiveSubscriptionIdentity {
        &self.preview_active_subscription_identity
    }

    pub fn preview_basis_identity(&self) -> &BridgeSubscriptionPreviewBasisIdentity {
        &self.preview_basis_identity
    }

    pub fn preview_scope_identity(&self) -> &BridgeSubscriptionPreviewScopeIdentity {
        &self.preview_scope_identity
    }

    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }

    pub fn preview_work_trace_identity(&self) -> &BridgeSubscriptionPreviewWorkTraceIdentity {
        &self.preview_work_trace_identity
    }

    pub fn preview_work_trace_digest(&self) -> &str {
        self.preview_work_trace_digest.as_ref()
    }

    pub fn residue_records(&self) -> &[BridgeSubscriptionPreviewLifecycleResidueRecord] {
        &self.residue_records
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
