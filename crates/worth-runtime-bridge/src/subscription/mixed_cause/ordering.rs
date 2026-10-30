use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::subscription::{
    BridgeSubscriptionCounters, BridgeSubscriptionMixedCauseDeniedCauseIdentity,
    BridgeSubscriptionMixedCauseOrderedCauseIdentity, BridgeSubscriptionMixedCauseOrderingIdentity,
    BridgeSubscriptionMixedCauseSuppressedCauseIdentity,
};

use super::comparison::{BridgeMixedCauseComparisonEvidence, Candidate};
use super::request::{BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest};
use super::{BridgeMixedCauseAsyncResultDisposition, BridgeMixedCauseAsyncResultTransition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMixedCauseOrderFamilyKind {
    TruthPatch,
    TemporalTruthPlusTime,
    TemporalTimeOnly,
    AsyncCompletion,
    AsyncDeniedCompletion,
    AsyncClassifiedDeniedCompletion,
    AsyncRetryLineage,
    AsyncRevalidationLineage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMixedCauseDeniedKind {
    AuthoritativePreviewCauseRejected,
    AsyncStaleCauseRejected,
    AsyncLineageNonDeliverable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMixedCauseSuppressedKind {
    DuplicateDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOrderedMixedCause {
    ordered_cause_identity: BridgeSubscriptionMixedCauseOrderedCauseIdentity,
    family_kind: BridgeMixedCauseOrderFamilyKind,
    source_identity: Arc<str>,
    source_digest: Arc<str>,
    order_ordinal: usize,
    comparison_evidence: BridgeMixedCauseComparisonEvidence,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSuppressedMixedCause {
    suppressed_cause_identity: BridgeSubscriptionMixedCauseSuppressedCauseIdentity,
    family_kind: BridgeMixedCauseOrderFamilyKind,
    source_identity: Arc<str>,
    source_digest: Arc<str>,
    suppressed_kind: BridgeMixedCauseSuppressedKind,
    suppressed_by_digest: Arc<str>,
    comparison_evidence: BridgeMixedCauseComparisonEvidence,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeDeniedMixedCause {
    denied_cause_identity: BridgeSubscriptionMixedCauseDeniedCauseIdentity,
    family_kind: BridgeMixedCauseOrderFamilyKind,
    source_identity: Arc<str>,
    source_digest: Arc<str>,
    denied_kind: BridgeMixedCauseDeniedKind,
    comparison_evidence: BridgeMixedCauseComparisonEvidence,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMixedCauseOrdering {
    ordering_identity: BridgeSubscriptionMixedCauseOrderingIdentity,
    lane_kind: BridgeMixedCauseOrderingLaneKind,
    ordered: Vec<BridgeOrderedMixedCause>,
    suppressed: Vec<BridgeSuppressedMixedCause>,
    denied: Vec<BridgeDeniedMixedCause>,
    async_result_transitions: Vec<BridgeMixedCauseAsyncResultTransition>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeMixedCauseOrdering {
    pub fn order(request: &BridgeMixedCauseOrderingRequest) -> Self {
        let mut candidates = request
            .inputs()
            .iter()
            .map(Candidate::from_input)
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));

        let mut seen = std::collections::BTreeMap::<String, String>::new();
        let mut exemplar_by_dedup = std::collections::BTreeMap::<String, Candidate>::new();
        let mut ordered = Vec::new();
        let mut suppressed = Vec::new();
        let mut denied = Vec::new();
        let mut async_result_transitions = Vec::new();
        let mut prior_ordered_candidate: Option<Candidate> = None;

        for candidate in candidates {
            if let Some(denied_kind) = candidate.denied_kind(request.lane_kind()) {
                if let Some(transition) = candidate.async_result_transition.as_ref() {
                    async_result_transitions.push(transition.admit(
                        BridgeMixedCauseAsyncResultDisposition::DeliveryDenied(denied_kind),
                    ));
                }
                denied.push(BridgeDeniedMixedCause::new(candidate, denied_kind));
                continue;
            }
            if let Some(first_digest) = seen.get(candidate.dedup_key.as_ref()) {
                if let Some(transition) = candidate.async_result_transition.as_ref() {
                    async_result_transitions.push(
                        transition
                            .admit(BridgeMixedCauseAsyncResultDisposition::DuplicateSuppressed),
                    );
                }
                suppressed.push(BridgeSuppressedMixedCause::new(
                    candidate.clone(),
                    BridgeMixedCauseSuppressedKind::DuplicateDigest,
                    Arc::from(first_digest.to_owned()),
                    exemplar_by_dedup
                        .get(candidate.dedup_key.as_ref())
                        .expect("duplicate suppression should retain exemplar"),
                ));
                continue;
            }

            let ordinal = ordered.len();
            if let Some(transition) = candidate.async_result_transition.as_ref() {
                async_result_transitions.push(
                    transition.admit(BridgeMixedCauseAsyncResultDisposition::Ordered { ordinal }),
                );
            }
            exemplar_by_dedup.insert(candidate.dedup_key.to_string(), candidate.clone());
            seen.insert(
                candidate.dedup_key.to_string(),
                candidate.source_digest.to_string(),
            );
            ordered.push(BridgeOrderedMixedCause::new(
                candidate.clone(),
                ordinal,
                prior_ordered_candidate.as_ref(),
            ));
            prior_ordered_candidate = Some(candidate);
        }

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-mixed-cause-ordering|lane={}|ordered={}|suppressed={}|denied={}",
            request.lane_kind().as_str(),
            ordered
                .iter()
                .map(BridgeOrderedMixedCause::digest)
                .collect::<Vec<_>>()
                .join(","),
            suppressed
                .iter()
                .map(BridgeSuppressedMixedCause::digest)
                .collect::<Vec<_>>()
                .join(","),
            denied
                .iter()
                .map(BridgeDeniedMixedCause::digest)
                .collect::<Vec<_>>()
                .join(","),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            ordering_identity: BridgeSubscriptionMixedCauseOrderingIdentity::admit_bridge_owned(
                format!("bridge-mixed-cause-ordering-id:sha256:{digest:x}"),
            ),
            lane_kind: request.lane_kind(),
            ordered,
            suppressed,
            denied,
            async_result_transitions,
            counters: BridgeSubscriptionCounters::from_mixed_cause_ordering(),
            canonical_basis,
            digest: Arc::from(format!("bridge-mixed-cause-ordering:sha256:{digest:x}")),
        }
    }

    pub fn ordering_identity(&self) -> &BridgeSubscriptionMixedCauseOrderingIdentity {
        &self.ordering_identity
    }

    pub fn lane_kind(&self) -> BridgeMixedCauseOrderingLaneKind {
        self.lane_kind
    }

    pub fn ordered(&self) -> &[BridgeOrderedMixedCause] {
        &self.ordered
    }

    pub fn suppressed(&self) -> &[BridgeSuppressedMixedCause] {
        &self.suppressed
    }

    pub fn denied(&self) -> &[BridgeDeniedMixedCause] {
        &self.denied
    }
    pub fn async_result_transitions(&self) -> &[BridgeMixedCauseAsyncResultTransition] {
        &self.async_result_transitions
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

impl BridgeOrderedMixedCause {
    fn new(candidate: Candidate, order_ordinal: usize, prior: Option<&Candidate>) -> Self {
        let comparison_evidence = match prior {
            Some(prior) => BridgeMixedCauseComparisonEvidence::ordered_after(&candidate, prior),
            None => BridgeMixedCauseComparisonEvidence::root(&candidate),
        };
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-ordered-mixed-cause|family={:?}|source={}|digest={}|ordinal={order_ordinal}|comparison={}",
            candidate.family_kind, candidate.source_identity, candidate.source_digest, comparison_evidence.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            ordered_cause_identity:
                BridgeSubscriptionMixedCauseOrderedCauseIdentity::admit_bridge_owned(format!(
                    "bridge-ordered-mixed-cause-id:sha256:{digest:x}"
                )),
            family_kind: candidate.family_kind,
            source_identity: candidate.source_identity,
            source_digest: candidate.source_digest,
            order_ordinal,
            comparison_evidence,
            counters: BridgeSubscriptionCounters::from_mixed_cause_ordered(),
            canonical_basis,
            digest: Arc::from(format!("bridge-ordered-mixed-cause:sha256:{digest:x}")),
        }
    }

    pub fn ordered_cause_identity(&self) -> &BridgeSubscriptionMixedCauseOrderedCauseIdentity {
        &self.ordered_cause_identity
    }

    pub fn family_kind(&self) -> BridgeMixedCauseOrderFamilyKind {
        self.family_kind
    }
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_ref()
    }
    pub fn source_digest(&self) -> &str {
        self.source_digest.as_ref()
    }
    pub fn order_ordinal(&self) -> usize {
        self.order_ordinal
    }
    pub fn comparison_evidence(&self) -> &BridgeMixedCauseComparisonEvidence {
        &self.comparison_evidence
    }
    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

impl BridgeSuppressedMixedCause {
    fn new(
        candidate: Candidate,
        suppressed_kind: BridgeMixedCauseSuppressedKind,
        suppressed_by_digest: Arc<str>,
        exemplar: &Candidate,
    ) -> Self {
        let comparison_evidence =
            BridgeMixedCauseComparisonEvidence::duplicate(&candidate, exemplar);
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-suppressed-mixed-cause|family={:?}|source={}|digest={}|kind={:?}|suppressed-by={}|comparison={}",
            candidate.family_kind, candidate.source_identity, candidate.source_digest, suppressed_kind, suppressed_by_digest, comparison_evidence.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            suppressed_cause_identity:
                BridgeSubscriptionMixedCauseSuppressedCauseIdentity::admit_bridge_owned(format!(
                    "bridge-suppressed-mixed-cause-id:sha256:{digest:x}"
                )),
            family_kind: candidate.family_kind,
            source_identity: candidate.source_identity,
            source_digest: candidate.source_digest,
            suppressed_kind,
            suppressed_by_digest,
            comparison_evidence,
            counters: BridgeSubscriptionCounters::from_mixed_cause_duplicate_suppression(),
            canonical_basis,
            digest: Arc::from(format!("bridge-suppressed-mixed-cause:sha256:{digest:x}")),
        }
    }

    pub fn suppressed_cause_identity(
        &self,
    ) -> &BridgeSubscriptionMixedCauseSuppressedCauseIdentity {
        &self.suppressed_cause_identity
    }

    pub fn suppressed_kind(&self) -> BridgeMixedCauseSuppressedKind {
        self.suppressed_kind
    }
    pub fn suppressed_by_digest(&self) -> &str {
        self.suppressed_by_digest.as_ref()
    }
    pub fn comparison_evidence(&self) -> &BridgeMixedCauseComparisonEvidence {
        &self.comparison_evidence
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

impl BridgeDeniedMixedCause {
    fn new(candidate: Candidate, denied_kind: BridgeMixedCauseDeniedKind) -> Self {
        let comparison_evidence =
            BridgeMixedCauseComparisonEvidence::denied(&candidate, denied_kind);
        let counters = match denied_kind {
            BridgeMixedCauseDeniedKind::AuthoritativePreviewCauseRejected => {
                BridgeSubscriptionCounters::from_mixed_cause_authoritative_preview_rejection()
            }
            BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected
            | BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable => {
                BridgeSubscriptionCounters::from_mixed_cause_denied()
            }
        };
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-denied-mixed-cause|family={:?}|source={}|digest={}|kind={:?}|comparison={}",
            candidate.family_kind,
            candidate.source_identity,
            candidate.source_digest,
            denied_kind,
            comparison_evidence.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            denied_cause_identity:
                BridgeSubscriptionMixedCauseDeniedCauseIdentity::admit_bridge_owned(format!(
                    "bridge-denied-mixed-cause-id:sha256:{digest:x}"
                )),
            family_kind: candidate.family_kind,
            source_identity: candidate.source_identity,
            source_digest: candidate.source_digest,
            denied_kind,
            comparison_evidence,
            counters,
            canonical_basis,
            digest: Arc::from(format!("bridge-denied-mixed-cause:sha256:{digest:x}")),
        }
    }

    pub fn denied_cause_identity(&self) -> &BridgeSubscriptionMixedCauseDeniedCauseIdentity {
        &self.denied_cause_identity
    }

    pub fn denied_kind(&self) -> BridgeMixedCauseDeniedKind {
        self.denied_kind
    }

    pub fn source_identity(&self) -> &str {
        self.source_identity.as_ref()
    }

    pub fn comparison_evidence(&self) -> &BridgeMixedCauseComparisonEvidence {
        &self.comparison_evidence
    }
    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
