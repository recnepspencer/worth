use super::super::BridgeCausalDigestRefusal;
use std::sync::Arc;

use worth_foundational::facade::{
    derive_canonical_digest, prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalDerivedDigest,
    CanonicalDigestAlgorithmId, CanonicalDigestFrontDoor, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::identity::{BridgeIdentity, BridgeIdentityEvidence};
use crate::routing::BridgeBulkPlanningFailure;

const RETAINED_CAUSAL_MAPPING_IDENTITY_DOMAIN: &str =
    "worth_runtime_bridge.causal_envelope.retained_mapping.identity";
const RETAINED_CAUSAL_MAPPING_IDENTITY_SCHEME: &str =
    "worth.runtime.bridge.retained-causal-mapping-identity.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedCausalMappingDigestArtifact {
    BulkPlanningCounters,
    BulkPlanningFailureRecord,
    BulkPlanningFailures,
    BulkPlanningRecord,
    ContinuityRecord,
    HistoricalEvaluationCounters,
    HistoricalEvaluationFailureRecord,
    HistoricalEvaluationRecord,
    MergeRecord,
    PreviewDiscardRecord,
    PreviewExecutionRecord,
    PreviewPromotionRecord,
    RouteRecord,
    SourceFailureRecord,
    SourceMaterializationRecord,
    StreamCheckpointRecord,
    StreamProtocolCounters,
    StreamReplayRecord,
    StructuralBranchComparisonRecord,
    StructuralRemapRecord,
    WritebackAdmissionRecord,
    WritebackExecutionRecord,
    WritebackMappedFamilyInput,
    WritebackMapperEnvelope,
    WritebackMapperRecord,
    WritebackReplayRecord,
}

impl RetainedCausalMappingDigestArtifact {
    pub(crate) fn digest_domain(self) -> &'static str {
        match self {
            Self::BulkPlanningCounters => "bridge-bulk-planning-counters",
            Self::BulkPlanningFailureRecord => {
                "bridge-causal-retained-bulk-planning-failure-record"
            }
            Self::BulkPlanningFailures => "bridge-bulk-planning-failures",
            Self::BulkPlanningRecord => "bridge-causal-retained-bulk-planning-record",
            Self::ContinuityRecord => "bridge-causal-retained-continuity-record",
            Self::HistoricalEvaluationCounters => "bridge-historical-evaluation-counters",
            Self::HistoricalEvaluationFailureRecord => {
                "bridge-causal-retained-historical-evaluation-failure-record"
            }
            Self::HistoricalEvaluationRecord => "bridge-causal-retained-historical-record",
            Self::MergeRecord => "bridge-causal-retained-merge-record",
            Self::PreviewDiscardRecord => "bridge-causal-retained-preview-discard-record",
            Self::PreviewExecutionRecord => "bridge-causal-retained-preview-execution-record",
            Self::PreviewPromotionRecord => "bridge-causal-retained-preview-promotion-record",
            Self::RouteRecord => "bridge-causal-retained-route-record",
            Self::SourceFailureRecord => "bridge-causal-retained-source-failure-record",
            Self::SourceMaterializationRecord => {
                "bridge-causal-retained-source-materialization-record"
            }
            Self::StreamCheckpointRecord => "bridge-causal-retained-stream-checkpoint-record",
            Self::StreamProtocolCounters => "bridge-stream-protocol-counters",
            Self::StreamReplayRecord => "bridge-causal-retained-stream-replay-record",
            Self::StructuralBranchComparisonRecord => {
                "bridge-causal-retained-structural-branch-comparison-record"
            }
            Self::StructuralRemapRecord => "bridge-causal-retained-structural-remap-record",
            Self::WritebackAdmissionRecord => "bridge-causal-retained-writeback-admission-record",
            Self::WritebackExecutionRecord => "bridge-causal-retained-writeback-execution-record",
            Self::WritebackMappedFamilyInput => {
                "bridge-causal-retained-writeback-mapped-family-input"
            }
            Self::WritebackMapperEnvelope => "bridge-causal-retained-writeback-mapper-envelope",
            Self::WritebackMapperRecord => "bridge-causal-retained-writeback-mapper-record",
            Self::WritebackReplayRecord => "bridge-causal-retained-writeback-replay-record",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedCausalMappingDigestBasis {
    entries: Arc<[RetainedCausalMappingDigestBasisEntry]>,
}

impl RetainedCausalMappingDigestBasis {
    pub(crate) fn from_counter_usizes(entries: impl IntoIterator<Item = usize>) -> Self {
        let entries = entries
            .into_iter()
            .map(RetainedCausalMappingDigestBasisEntry::from_counter)
            .collect::<Vec<_>>();
        Self {
            entries: Arc::from(entries),
        }
    }

    pub(crate) fn from_bulk_planning_failure_records(
        failures: &[BridgeBulkPlanningFailure],
    ) -> Result<Self, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal> {
        let entries = failures
            .iter()
            .map(|failure| {
                bulk_planning_failure_evidence_identity(failure)
                    .map(RetainedCausalMappingDigestBasisEntry::from_evidence)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            entries: Arc::from(entries),
        })
    }

    fn entries(&self) -> &[RetainedCausalMappingDigestBasisEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetainedCausalMappingDigestBasisEntry {
    part: RetainedCausalMappingIdentityPart,
}

impl RetainedCausalMappingDigestBasisEntry {
    fn from_counter(value: usize) -> Self {
        Self {
            part: retained_mapping_counter_part(value),
        }
    }

    fn from_evidence(identity: BridgeIdentityEvidence) -> Self {
        Self {
            part: retained_mapping_evidence_part(identity),
        }
    }

    fn part(&self) -> RetainedCausalMappingIdentityPart {
        self.part.as_borrowed()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RetainedCausalMappingIdentityPart {
    Evidence(BridgeIdentityEvidence),
    Shape(Arc<str>),
    Counter(usize),
}

impl RetainedCausalMappingIdentityPart {
    fn as_borrowed(&self) -> RetainedCausalMappingIdentityPart {
        self.clone()
    }

    fn value(&self) -> String {
        match self {
            Self::Evidence(identity) => identity.as_str().to_string(),
            Self::Shape(value) => value.as_ref().to_string(),
            Self::Counter(value) => value.to_string(),
        }
    }

    fn canonical_kind(&self) -> CanonicalBasisEntryKind {
        match self {
            Self::Evidence(_) => CanonicalBasisEntryKind::Identity,
            Self::Shape(_) => CanonicalBasisEntryKind::Shape,
            Self::Counter(_) => CanonicalBasisEntryKind::Value,
        }
    }
}

pub(crate) fn bulk_planning_failure_evidence_identity(
    failure: &BridgeBulkPlanningFailure,
) -> Result<BridgeIdentityEvidence, BridgeCausalDigestRefusal> {
    use crate::routing::planning::planning_failure_kind_label;

    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::BulkPlanningFailureRecord,
        &[
            retained_mapping_shape_part(planning_failure_kind_label(failure.kind())),
            retained_mapping_shape_part(failure.boundary()),
            retained_mapping_shape_part(failure.detail()),
        ],
    )?)
}

pub(crate) fn retained_mapping_evidence_part(
    identity: BridgeIdentityEvidence,
) -> RetainedCausalMappingIdentityPart {
    RetainedCausalMappingIdentityPart::Evidence(identity)
}

pub(crate) fn retained_mapping_bridge_identity_part<T>(
    identity: &BridgeIdentity<T>,
) -> RetainedCausalMappingIdentityPart {
    retained_mapping_evidence_part(identity.bridge_admission_evidence())
}

pub(crate) fn retained_mapping_counter_part(value: usize) -> RetainedCausalMappingIdentityPart {
    RetainedCausalMappingIdentityPart::Counter(value)
}

pub(crate) fn retained_mapping_shape_part(
    value: impl AsRef<str>,
) -> RetainedCausalMappingIdentityPart {
    RetainedCausalMappingIdentityPart::Shape(Arc::from(value.as_ref()))
}

pub(crate) fn compose_retained_causal_mapping_evidence_identity(
    artifact: RetainedCausalMappingDigestArtifact,
    parts: &[RetainedCausalMappingIdentityPart],
) -> Result<BridgeIdentityEvidence, BridgeCausalDigestRefusal> {
    Ok(compose_retained_causal_mapping_evidence_identity_for_parts(
        artifact,
        parts
            .iter()
            .map(RetainedCausalMappingIdentityPart::as_borrowed),
    )?)
}

pub(crate) fn compose_retained_causal_mapping_evidence_identity_for_basis(
    artifact: RetainedCausalMappingDigestArtifact,
    basis: &RetainedCausalMappingDigestBasis,
) -> Result<BridgeIdentityEvidence, BridgeCausalDigestRefusal> {
    Ok(compose_retained_causal_mapping_evidence_identity_for_parts(
        artifact,
        basis.entries().iter().map(|entry| entry.part()),
    )?)
}

fn compose_retained_causal_mapping_evidence_identity_for_parts(
    artifact: RetainedCausalMappingDigestArtifact,
    parts: impl IntoIterator<Item = RetainedCausalMappingIdentityPart>,
) -> Result<BridgeIdentityEvidence, BridgeCausalDigestRefusal> {
    let mut count = 0usize;
    let mut entries = vec![
        retained_mapping_entry(
            "bridge.retained.scheme",
            CanonicalBasisEntryKind::Header,
            RETAINED_CAUSAL_MAPPING_IDENTITY_SCHEME,
        ),
        retained_mapping_entry(
            "bridge.retained.artifact",
            CanonicalBasisEntryKind::Header,
            artifact.digest_domain(),
        ),
    ];
    for (index, part) in parts.into_iter().enumerate() {
        entries.push(retained_mapping_entry(
            sequence_locus(index),
            part.canonical_kind(),
            part.value().as_str(),
        ));
        count = index + 1;
    }
    entries.push(retained_mapping_entry(
        "bridge.retained.part.count",
        CanonicalBasisEntryKind::Shape,
        count.to_string(),
    ));
    let digest = derive_canonical_retained_mapping_identity(entries).map_err(|cause| {
        BridgeCausalDigestRefusal {
            subject: artifact.digest_domain(),
            cause,
        }
    })?;
    Ok(BridgeIdentityEvidence::from_canonical_bridge_evidence(
        canonical_retained_mapping_identity_token(&digest),
        artifact.digest_domain(),
    ))
}

fn sequence_locus(index: usize) -> String {
    let mut locus = String::from("bridge.retained.part.");
    locus.push_str(&index.to_string());
    locus
}

fn retained_mapping_entry(
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: impl Into<String>,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Future(RETAINED_CAUSAL_MAPPING_IDENTITY_DOMAIN),
        CanonicalBasisLocus::Named(locus.into().into()),
        kind,
        CanonicalBasisValue::ExactText(value.into().into()),
    )
}

fn derive_canonical_retained_mapping_identity(
    entries: Vec<CanonicalBasisEntry>,
) -> Result<CanonicalDerivedDigest, worth_foundational::facade::CanonicalDigestDerivationDenial> {
    let version = CanonicalizationRuleVersion::new(RETAINED_CAUSAL_MAPPING_IDENTITY_SCHEME)
        .expect("retained causal mapping evidence scheme must remain canonical");
    let ready = retained_mapping_basis_from_entries(version.clone(), entries);
    let bundle = match prepare_canonical_basis_bundle(version, [ready]) {
        TransitionOutcome::Success(bundle) => bundle,
        outcome => {
            panic!("retained causal mapping identity bundle should prepare cleanly: {outcome:?}")
        }
    };
    let digest_ready =
        match CanonicalDigestFrontDoor.for_bundle(bundle, CanonicalDigestAlgorithmId::sha256()) {
            TransitionOutcome::Success(ready) => ready,
            TransitionOutcome::Denied(cause) => return Err(cause),
            _ => unreachable!("canonical digest admission only succeeds or denies"),
        };
    Ok(derive_canonical_digest(digest_ready))
}

fn retained_mapping_basis_from_entries(
    version: CanonicalizationRuleVersion,
    entries: Vec<CanonicalBasisEntry>,
) -> CanonicalBasisReadyArtifact {
    match prepare_canonical_basis_sequence(
        version,
        CanonicalBasisDomain::Future(RETAINED_CAUSAL_MAPPING_IDENTITY_DOMAIN),
        entries,
    ) {
        TransitionOutcome::Success(ready) => ready,
        outcome => panic!("retained causal mapping basis should prepare cleanly: {outcome:?}"),
    }
}

fn canonical_retained_mapping_identity_token(digest: &CanonicalDerivedDigest) -> String {
    use std::fmt::Write;

    let mut token = String::from(RETAINED_CAUSAL_MAPPING_IDENTITY_SCHEME);
    token.push(':');
    token.push_str(digest.metadata().algorithm().id().as_str());
    token.push(':');
    for byte in digest.value().bytes() {
        write!(&mut token, "{byte:02x}").expect("writing to String cannot fail");
    }
    token
}
