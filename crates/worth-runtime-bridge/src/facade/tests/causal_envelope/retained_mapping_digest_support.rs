use crate::diagnostics::causal_envelope::retained_mapping::digest_basis::{
    compose_retained_causal_mapping_evidence_identity, retained_mapping_evidence_part,
    retained_mapping_shape_part, RetainedCausalMappingDigestArtifact,
    RetainedCausalMappingIdentityPart,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Copy, Debug)]
pub(super) enum ExpectedRetainedCausalDigestArtifact {
    ContinuityRecord,
    HistoricalEvaluationRecord,
    MergeRecord,
    PreviewDiscardRecord,
    PreviewExecutionRecord,
    PreviewPromotionRecord,
    RouteRecord,
    SourceMaterializationRecord,
    StreamReplayRecord,
    StructuralBranchComparisonRecord,
    StructuralRemapRecord,
}

impl From<ExpectedRetainedCausalDigestArtifact> for RetainedCausalMappingDigestArtifact {
    fn from(value: ExpectedRetainedCausalDigestArtifact) -> Self {
        match value {
            ExpectedRetainedCausalDigestArtifact::ContinuityRecord => Self::ContinuityRecord,
            ExpectedRetainedCausalDigestArtifact::HistoricalEvaluationRecord => {
                Self::HistoricalEvaluationRecord
            }
            ExpectedRetainedCausalDigestArtifact::MergeRecord => Self::MergeRecord,
            ExpectedRetainedCausalDigestArtifact::PreviewDiscardRecord => {
                Self::PreviewDiscardRecord
            }
            ExpectedRetainedCausalDigestArtifact::PreviewExecutionRecord => {
                Self::PreviewExecutionRecord
            }
            ExpectedRetainedCausalDigestArtifact::PreviewPromotionRecord => {
                Self::PreviewPromotionRecord
            }
            ExpectedRetainedCausalDigestArtifact::RouteRecord => Self::RouteRecord,
            ExpectedRetainedCausalDigestArtifact::SourceMaterializationRecord => {
                Self::SourceMaterializationRecord
            }
            ExpectedRetainedCausalDigestArtifact::StreamReplayRecord => Self::StreamReplayRecord,
            ExpectedRetainedCausalDigestArtifact::StructuralBranchComparisonRecord => {
                Self::StructuralBranchComparisonRecord
            }
            ExpectedRetainedCausalDigestArtifact::StructuralRemapRecord => {
                Self::StructuralRemapRecord
            }
        }
    }
}

pub(super) fn expected_retained_causal_digest(
    artifact: ExpectedRetainedCausalDigestArtifact,
    parts: &[&str],
) -> String {
    let typed_parts = parts
        .iter()
        .enumerate()
        .map(|(index, part)| expected_part_kind(artifact, index, part))
        .collect::<Vec<_>>();
    compose_retained_causal_mapping_evidence_identity(artifact.into(), &typed_parts)
        .expect("bounded digest fixture")
        .as_str()
        .to_string()
}

fn expected_part_kind(
    artifact: ExpectedRetainedCausalDigestArtifact,
    index: usize,
    part: &str,
) -> RetainedCausalMappingIdentityPart {
    if expected_bridge_identity_index(artifact, index) {
        return expected_bridge_identity_part(part);
    }
    if expected_shape_index(artifact, index) {
        return retained_mapping_shape_part(part);
    }
    panic!(
        "unexpected retained mapping part index {index} for {artifact:?}: production paths no longer ingest external digest labels as shape authority"
    );
}

fn expected_bridge_identity_part(part: &str) -> RetainedCausalMappingIdentityPart {
    retained_mapping_evidence_part(
        BridgeIdentityEvidence::from_bridge_owner_external_authority(part),
    )
}

fn expected_bridge_identity_index(
    artifact: ExpectedRetainedCausalDigestArtifact,
    index: usize,
) -> bool {
    match artifact {
        ExpectedRetainedCausalDigestArtifact::ContinuityRecord => matches!(index, 0 | 2 | 3),
        ExpectedRetainedCausalDigestArtifact::HistoricalEvaluationRecord => {
            matches!(index, 0..=2)
        }
        ExpectedRetainedCausalDigestArtifact::MergeRecord => matches!(index, 0 | 2),
        ExpectedRetainedCausalDigestArtifact::PreviewDiscardRecord => matches!(index, 0 | 1),
        ExpectedRetainedCausalDigestArtifact::PreviewExecutionRecord => index == 0,
        ExpectedRetainedCausalDigestArtifact::PreviewPromotionRecord => matches!(index, 0 | 1),
        ExpectedRetainedCausalDigestArtifact::RouteRecord => matches!(index, 0..=2),
        ExpectedRetainedCausalDigestArtifact::SourceMaterializationRecord => index == 0,
        ExpectedRetainedCausalDigestArtifact::StreamReplayRecord => matches!(index, 0..=2),
        ExpectedRetainedCausalDigestArtifact::StructuralBranchComparisonRecord
        | ExpectedRetainedCausalDigestArtifact::StructuralRemapRecord => matches!(index, 0 | 2),
    }
}

fn expected_shape_index(artifact: ExpectedRetainedCausalDigestArtifact, index: usize) -> bool {
    match artifact {
        ExpectedRetainedCausalDigestArtifact::ContinuityRecord
        | ExpectedRetainedCausalDigestArtifact::MergeRecord
        | ExpectedRetainedCausalDigestArtifact::StructuralBranchComparisonRecord
        | ExpectedRetainedCausalDigestArtifact::StructuralRemapRecord => index == 1,
        ExpectedRetainedCausalDigestArtifact::StreamReplayRecord => index == 3,
        ExpectedRetainedCausalDigestArtifact::HistoricalEvaluationRecord
        | ExpectedRetainedCausalDigestArtifact::PreviewDiscardRecord
        | ExpectedRetainedCausalDigestArtifact::PreviewExecutionRecord
        | ExpectedRetainedCausalDigestArtifact::PreviewPromotionRecord
        | ExpectedRetainedCausalDigestArtifact::RouteRecord
        | ExpectedRetainedCausalDigestArtifact::SourceMaterializationRecord => false,
    }
}
