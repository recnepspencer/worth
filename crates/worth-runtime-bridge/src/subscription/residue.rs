use std::sync::Arc;

use sha2::{Digest, Sha256};

mod proof;
mod rejection;

pub use proof::BridgeSubscriptionPreviewDiscardResidueProof;
pub use rejection::{
    BridgeSubscriptionPreviewDiscardResidueRejection,
    BridgeSubscriptionPreviewDiscardResidueRejectionContext,
    BridgeSubscriptionPreviewDiscardResidueRejectionKind,
    BridgeSubscriptionPreviewResidueCategoryCount,
};

use super::{
    BridgePreviewActiveSubscription, BridgePreviewActiveSubscriptionIdentity,
    BridgeSubscriptionCounters, BridgeSubscriptionPreviewResidueArtifactIdentity,
    BridgeSubscriptionPreviewResidueScopeIdentity,
    BridgeSubscriptionPreviewResidueScopeIndexIdentity, BridgeSubscriptionPreviewWorkKind,
    BridgeSubscriptionPreviewWorkTrace,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeSubscriptionPreviewResidueCategory {
    AuthoritativeTruthSubscription,
    BridgeSubscriptionRegistry,
    ActiveDelivery,
    FanoutConsumerContract,
    Continuation,
    CheckpointReplay,
    SignalVisible,
}

impl BridgeSubscriptionPreviewResidueCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthoritativeTruthSubscription => "authoritative_truth_subscription",
            Self::BridgeSubscriptionRegistry => "bridge_subscription_registry",
            Self::ActiveDelivery => "active_delivery",
            Self::FanoutConsumerContract => "fanout_consumer_contract",
            Self::Continuation => "continuation",
            Self::CheckpointReplay => "checkpoint_replay",
            Self::SignalVisible => "signal_visible",
        }
    }

    pub(super) const fn all() -> [Self; 7] {
        [
            Self::AuthoritativeTruthSubscription,
            Self::BridgeSubscriptionRegistry,
            Self::ActiveDelivery,
            Self::FanoutConsumerContract,
            Self::Continuation,
            Self::CheckpointReplay,
            Self::SignalVisible,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewResidueArtifactInput {
    category: BridgeSubscriptionPreviewResidueCategory,
    residue_count: usize,
    evidence_digest: Arc<str>,
}

impl BridgeSubscriptionPreviewResidueArtifactInput {
    pub fn from_preview_work_trace(
        category: BridgeSubscriptionPreviewResidueCategory,
        residue_count: usize,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
    ) -> Self {
        let evidence_digest =
            preview_residue_evidence_digest_from_work_trace(category, preview_work_trace);
        Self {
            category,
            residue_count,
            evidence_digest,
        }
    }

    pub fn zero_from_preview_work_trace(
        category: BridgeSubscriptionPreviewResidueCategory,
        preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
    ) -> Self {
        Self::from_preview_work_trace(category, 0, preview_work_trace)
    }

    pub fn category(&self) -> BridgeSubscriptionPreviewResidueCategory {
        self.category
    }

    pub fn evidence_digest(&self) -> &str {
        self.evidence_digest.as_ref()
    }
}

fn preview_residue_evidence_digest_from_work_trace(
    category: BridgeSubscriptionPreviewResidueCategory,
    preview_work_trace: &BridgeSubscriptionPreviewWorkTrace,
) -> Arc<str> {
    let record_digest = match category {
        BridgeSubscriptionPreviewResidueCategory::AuthoritativeTruthSubscription
        | BridgeSubscriptionPreviewResidueCategory::BridgeSubscriptionRegistry => {
            preview_work_trace.record_digest_for(BridgeSubscriptionPreviewWorkKind::Routing)
        }
        BridgeSubscriptionPreviewResidueCategory::ActiveDelivery
        | BridgeSubscriptionPreviewResidueCategory::FanoutConsumerContract => {
            preview_work_trace.record_digest_for(BridgeSubscriptionPreviewWorkKind::Delivery)
        }
        BridgeSubscriptionPreviewResidueCategory::Continuation
        | BridgeSubscriptionPreviewResidueCategory::CheckpointReplay => {
            preview_work_trace.record_digest_for(BridgeSubscriptionPreviewWorkKind::Continuation)
        }
        BridgeSubscriptionPreviewResidueCategory::SignalVisible => {
            preview_work_trace.record_digest_for(BridgeSubscriptionPreviewWorkKind::Diagnostics)
        }
    };
    Arc::from(format!(
        "preview-work-zero-residue|trace={}|scope={}|record={record_digest}|category={}",
        preview_work_trace.digest(),
        preview_work_trace.preview_residue_scope_identity().as_str(),
        category.as_str(),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionPreviewResidueArtifactRecord {
    artifact_identity: BridgeSubscriptionPreviewResidueArtifactIdentity,
    preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    category: BridgeSubscriptionPreviewResidueCategory,
    residue_count: usize,
    evidence_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewResidueArtifactRecord {
    fn from_input(
        preview_residue_scope_identity: &BridgeSubscriptionPreviewResidueScopeIdentity,
        input: BridgeSubscriptionPreviewResidueArtifactInput,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-preview-residue-artifact|scope={}|category={}|residue-count={}|evidence={}",
            preview_residue_scope_identity.as_str(),
            input.category.as_str(),
            input.residue_count,
            input.evidence_digest.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            artifact_identity: BridgeSubscriptionPreviewResidueArtifactIdentity::admit_bridge_owned(
                format!("bridge-subscription-preview-residue-artifact-id:sha256:{digest:x}"),
            ),
            preview_residue_scope_identity: preview_residue_scope_identity.clone(),
            category: input.category,
            residue_count: input.residue_count,
            evidence_digest: input.evidence_digest,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-preview-residue-artifact:sha256:{digest:x}"
            )),
        }
    }

    pub fn artifact_identity(&self) -> &BridgeSubscriptionPreviewResidueArtifactIdentity {
        &self.artifact_identity
    }

    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }

    pub fn category(&self) -> BridgeSubscriptionPreviewResidueCategory {
        self.category
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
pub struct BridgeSubscriptionPreviewResidueScopeIndex {
    preview_residue_scope_index_identity: BridgeSubscriptionPreviewResidueScopeIndexIdentity,
    preview_active_subscription_identity: BridgePreviewActiveSubscriptionIdentity,
    preview_residue_scope_identity: BridgeSubscriptionPreviewResidueScopeIdentity,
    artifact_records: Arc<[BridgeSubscriptionPreviewResidueArtifactRecord]>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionPreviewResidueScopeIndex {
    pub(crate) fn build(
        preview_active: &BridgePreviewActiveSubscription,
        artifact_inputs: Vec<BridgeSubscriptionPreviewResidueArtifactInput>,
    ) -> Self {
        let mut artifact_records = artifact_inputs
            .into_iter()
            .map(|input| {
                BridgeSubscriptionPreviewResidueArtifactRecord::from_input(
                    preview_active.preview_residue_scope_identity(),
                    input,
                )
            })
            .collect::<Vec<_>>();
        artifact_records.sort_by(|left, right| {
            left.category()
                .cmp(&right.category())
                .then_with(|| left.digest().cmp(right.digest()))
        });
        let artifact_digest_list = artifact_records
            .iter()
            .map(BridgeSubscriptionPreviewResidueArtifactRecord::digest)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-preview-residue-scope-index|preview-active={}|scope={}|artifacts={}",
            preview_active.preview_active_subscription_identity().as_str(),
            preview_active.preview_residue_scope_identity().as_str(),
            artifact_digest_list,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            preview_residue_scope_index_identity:
                BridgeSubscriptionPreviewResidueScopeIndexIdentity::admit_bridge_owned(format!(
                    "bridge-subscription-preview-residue-scope-index-id:sha256:{digest:x}"
                )),
            preview_active_subscription_identity: preview_active
                .preview_active_subscription_identity()
                .clone(),
            preview_residue_scope_identity: preview_active.preview_residue_scope_identity().clone(),
            counters: BridgeSubscriptionCounters::from_subscription_preview_residue_scope_index(
                artifact_records.len(),
            ),
            artifact_records: Arc::from(artifact_records),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-preview-residue-scope-index:sha256:{digest:x}"
            )),
        }
    }

    pub fn preview_residue_scope_index_identity(
        &self,
    ) -> &BridgeSubscriptionPreviewResidueScopeIndexIdentity {
        &self.preview_residue_scope_index_identity
    }

    pub fn preview_active_subscription_identity(&self) -> &BridgePreviewActiveSubscriptionIdentity {
        &self.preview_active_subscription_identity
    }

    pub fn preview_residue_scope_identity(&self) -> &BridgeSubscriptionPreviewResidueScopeIdentity {
        &self.preview_residue_scope_identity
    }

    pub fn artifact_records(&self) -> &[BridgeSubscriptionPreviewResidueArtifactRecord] {
        &self.artifact_records
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
