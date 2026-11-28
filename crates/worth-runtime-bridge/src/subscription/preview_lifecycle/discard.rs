use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionPreviewLifecycleResidueEnvelope,
    BridgeSubscriptionPreviewLifecycleResidueKind, BridgeSubscriptionPreviewLifecycleResidueRecord,
};
use crate::subscription::{
    BridgePreviewActiveSubscription, BridgePreviewActiveSubscriptionIdentity,
    BridgeSubscriptionCounters, BridgeSubscriptionPreviewDiscardResidueProofIdentity,
    BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity,
    BridgeSubscriptionPreviewResidueScopeIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleResidueKindCount {
    kind: BridgeSubscriptionPreviewLifecycleResidueKind,
    residue_count: usize,
}

impl BridgeSubscriptionPreviewLifecycleResidueKindCount {
    fn new(kind: BridgeSubscriptionPreviewLifecycleResidueKind, residue_count: usize) -> Self {
        Self {
            kind,
            residue_count,
        }
    }

    pub fn kind(&self) -> BridgeSubscriptionPreviewLifecycleResidueKind {
        self.kind
    }

    pub fn residue_count(&self) -> usize {
        self.residue_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionPreviewLifecycleDiscardRejectionKind {
    PreviewActiveMismatch,
    PreviewResidueScopeMismatch,
    MissingResidueKind,
    DuplicateResidueKind,
    NonzeroResidue,
}

impl BridgeSubscriptionPreviewLifecycleDiscardRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreviewActiveMismatch => "preview_active_mismatch",
            Self::PreviewResidueScopeMismatch => "preview_residue_scope_mismatch",
            Self::MissingResidueKind => "missing_residue_kind",
            Self::DuplicateResidueKind => "duplicate_residue_kind",
            Self::NonzeroResidue => "nonzero_residue",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeSubscriptionPreviewLifecycleDiscardRejectionContext {
    PreviewActiveMismatch {
        preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
        envelope_preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
    },
    PreviewResidueScopeMismatch {
        preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
        envelope_preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    },
    MissingResidueKind(BridgeSubscriptionPreviewLifecycleResidueKind),
    DuplicateResidueKind(BridgeSubscriptionPreviewLifecycleResidueKind),
    NonzeroResidue(Arc<[BridgeSubscriptionPreviewLifecycleResidueKindCount]>),
}

impl BridgeSubscriptionPreviewLifecycleDiscardRejectionContext {
    pub fn nonzero_kinds(&self) -> &[BridgeSubscriptionPreviewLifecycleResidueKindCount] {
        match self {
            Self::NonzeroResidue(kinds) => kinds.as_ref(),
            _ => &[],
        }
    }

    fn canonical_basis(&self) -> String {
        match self {
            Self::PreviewActiveMismatch {
                preview_active_subscription_identity,
                envelope_preview_active_subscription_identity,
            } => format!(
                "preview-active={}|envelope-preview-active={}",
                preview_active_subscription_identity.as_str(),
                envelope_preview_active_subscription_identity.as_str(),
            ),
            Self::PreviewResidueScopeMismatch {
                preview_residue_scope_identity,
                envelope_preview_residue_scope_identity,
            } => format!(
                "preview-scope={}|envelope-scope={}",
                preview_residue_scope_identity.as_str(),
                envelope_preview_residue_scope_identity.as_str(),
            ),
            Self::MissingResidueKind(kind) => format!("missing-kind={}", kind.as_str()),
            Self::DuplicateResidueKind(kind) => format!("duplicate-kind={}", kind.as_str()),
            Self::NonzeroResidue(kinds) => format!(
                "nonzero={}",
                kinds
                    .iter()
                    .map(|kind| format!("{}={}", kind.kind().as_str(), kind.residue_count()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleDiscardRejection {
    rejection_kind: BridgeSubscriptionPreviewLifecycleDiscardRejectionKind,
    rejection_context: BridgeSubscriptionPreviewLifecycleDiscardRejectionContext,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleDiscardRejection {
    fn new(
        rejection_kind: BridgeSubscriptionPreviewLifecycleDiscardRejectionKind,
        rejection_context: BridgeSubscriptionPreviewLifecycleDiscardRejectionContext,
        nonzero_residue: bool,
        residue_check_count: usize,
    ) -> Self {
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-discard-rejection|kind={}|context={}",
            rejection_kind.as_str(),
            rejection_context.canonical_basis(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind,
            rejection_context,
            counters: BridgeSubscriptionCounters::from_subscription_preview_discard_rejection(
                nonzero_residue,
                residue_check_count,
            ),
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-discard-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionPreviewLifecycleDiscardRejectionKind {
        self.rejection_kind
    }

    pub fn rejection_context(&self) -> &BridgeSubscriptionPreviewLifecycleDiscardRejectionContext {
        &self.rejection_context
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewLifecycleDiscardProof {
    proof_identity: BridgeSubscriptionPreviewDiscardResidueProofIdentity,
    preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
    residue_envelope_identity: BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity,
    preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    residue_records: Arc<[BridgeSubscriptionPreviewLifecycleResidueRecord]>,
    total_residue_count: usize,
    counters: BridgeSubscriptionCounters,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewLifecycleDiscardProof {
    pub(crate) fn prove(
        preview_active: BridgePreviewActiveSubscription,
        residue_envelope: BridgeSubscriptionPreviewLifecycleResidueEnvelope,
    ) -> Result<
        Self,
        (
            BridgePreviewActiveSubscription,
            BridgeSubscriptionPreviewLifecycleDiscardRejection,
        ),
    > {
        if residue_envelope.preview_active_subscription_identity()
            != preview_active.preview_active_subscription_identity()
        {
            let denial = BridgeSubscriptionPreviewLifecycleDiscardRejection::new(
                BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::PreviewActiveMismatch,
                BridgeSubscriptionPreviewLifecycleDiscardRejectionContext::PreviewActiveMismatch {
                    preview_active_subscription_identity: preview_active
                        .preview_active_subscription_identity()
                        .clone(),
                    envelope_preview_active_subscription_identity: residue_envelope
                        .preview_active_subscription_identity()
                        .clone(),
                },
                false,
                0,
            );
            return Err((preview_active, denial));
        }
        if residue_envelope.preview_residue_scope_identity()
            != preview_active.preview_residue_scope_identity()
        {
            let denial = BridgeSubscriptionPreviewLifecycleDiscardRejection::new(
                BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::PreviewResidueScopeMismatch,
                BridgeSubscriptionPreviewLifecycleDiscardRejectionContext::PreviewResidueScopeMismatch {
                    preview_residue_scope_identity: preview_active
                        .preview_residue_scope_identity()
                        .clone(),
                    envelope_preview_residue_scope_identity: residue_envelope
                        .preview_residue_scope_identity()
                        .clone(),
                },
                false,
                0,
            );
            return Err((preview_active, denial));
        }

        let residue_check_count = residue_envelope.residue_records().len();
        let mut counts = BTreeMap::<BridgeSubscriptionPreviewLifecycleResidueKind, usize>::new();
        let mut seen = BTreeSet::<BridgeSubscriptionPreviewLifecycleResidueKind>::new();
        for record in residue_envelope.residue_records() {
            if !seen.insert(record.kind()) {
                let denial = BridgeSubscriptionPreviewLifecycleDiscardRejection::new(
                    BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::DuplicateResidueKind,
                    BridgeSubscriptionPreviewLifecycleDiscardRejectionContext::DuplicateResidueKind(
                        record.kind(),
                    ),
                    false,
                    residue_check_count,
                );
                return Err((preview_active, denial));
            }
            *counts.entry(record.kind()).or_default() += record.residue_count();
        }
        for required in BridgeSubscriptionPreviewLifecycleResidueKind::all() {
            if !seen.contains(&required) {
                let denial = BridgeSubscriptionPreviewLifecycleDiscardRejection::new(
                    BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::MissingResidueKind,
                    BridgeSubscriptionPreviewLifecycleDiscardRejectionContext::MissingResidueKind(
                        required,
                    ),
                    false,
                    residue_check_count,
                );
                return Err((preview_active, denial));
            }
        }

        let total_residue_count = counts.values().sum::<usize>();
        if total_residue_count != 0 {
            let nonzero = counts
                .iter()
                .filter(|&(_kind, count)| *count != 0)
                .map(|(kind, count)| {
                    BridgeSubscriptionPreviewLifecycleResidueKindCount::new(*kind, *count)
                })
                .collect::<Vec<_>>();
            let denial = BridgeSubscriptionPreviewLifecycleDiscardRejection::new(
                BridgeSubscriptionPreviewLifecycleDiscardRejectionKind::NonzeroResidue,
                BridgeSubscriptionPreviewLifecycleDiscardRejectionContext::NonzeroResidue(
                    Arc::from(nonzero),
                ),
                true,
                residue_check_count,
            );
            return Err((preview_active, denial));
        }

        let record_digests = residue_envelope
            .residue_records()
            .iter()
            .map(BridgeSubscriptionPreviewLifecycleResidueRecord::digest)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = format!(
            "bridge-subscription-preview-lifecycle-discard-proof|preview-active={}|residue-envelope={}|residue-scope={}|records={}|total-residue={}",
            preview_active.preview_active_subscription_identity().as_str(),
            residue_envelope.residue_envelope_identity().as_str(),
            preview_active.preview_residue_scope_identity().as_str(),
            record_digests,
            total_residue_count,
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            proof_identity:
                BridgeSubscriptionPreviewDiscardResidueProofIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-lifecycle-discard-proof-id:sha256:{digest:x}"
                )),
            preview_active_subscription_identity: preview_active
                .preview_active_subscription_identity()
                .clone(),
            residue_envelope_identity: residue_envelope.residue_envelope_identity().clone(),
            preview_residue_scope_identity: preview_active.preview_residue_scope_identity().clone(),
            residue_records: Arc::from(residue_envelope.residue_records().to_vec()),
            total_residue_count,
            counters: BridgeSubscriptionCounters::from_subscription_preview_discard(
                residue_check_count,
            ),
            digest: Arc::from(format!(
                "bridge-subscription-preview-lifecycle-discard-proof:sha256:{digest:x}"
            )),
        })
    }

    pub fn proof_identity(&self) -> &BridgeSubscriptionPreviewDiscardResidueProofIdentity {
        &self.proof_identity
    }

    pub fn preview_active_subscription_identity(&self) -> &BridgePreviewActiveSubscriptionIdentity {
        &self.preview_active_subscription_identity
    }

    pub fn residue_envelope_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewLifecycleResidueEnvelopeIdentity {
        &self.residue_envelope_identity
    }

    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }

    pub fn residue_records(&self) -> &[BridgeSubscriptionPreviewLifecycleResidueRecord] {
        &self.residue_records
    }

    pub fn total_residue_count(&self) -> usize {
        self.total_residue_count
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }
}
