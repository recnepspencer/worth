use worth_runtime_bridge::facade::{
    BridgeHistoricalEvaluationDecisionLog, BridgeHistoricalMaterializationPath,
    BridgeTruthViewKind, BridgeTruthViewPolicyResolution, HistoricalEvaluationDeclaration,
    LoweredHistoricalEvaluationArtifact, ResolvedTruthViewPolicy, SourceMaterializationRecord,
    TruthViewReplayContinuity, TruthViewRetentionAdmission, TruthViewSourceCapability,
};

use crate::basis::{BasisResolutionMode, ResolvedBasisProof};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::contracts::HistoricalPathReuseDescriptor;
use super::error::HistoricalEvaluationError;
use super::path_classes::{
    AdmittedHistoricalPathClass, RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};
use super::request::{HistoricalCapabilityDescriptor, HistoricalMaterializationDescriptor};

pub(crate) fn lower_policy_resolution(
    declaration: &HistoricalEvaluationDeclaration,
    resolution: &BridgeTruthViewPolicyResolution,
    source_record: Option<&SourceMaterializationRecord>,
    requested_path_class: &RequestedHistoricalPathClass,
) -> Result<HistoricalCapabilityDescriptor, HistoricalEvaluationError> {
    match resolution {
        BridgeTruthViewPolicyResolution::Admitted(policy) => {
            Ok(lower_admitted_policy(
                declaration,
                policy,
                source_record,
                requested_path_class,
            ))
        }
        BridgeTruthViewPolicyResolution::Rejected(rejection) => Err(
            HistoricalEvaluationError::UnsupportedHistoricalPathRequest {
                requested_path_class: requested_path_class.clone(),
                reason: match rejection.kind() {
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::ReplayNotPermitted => {
                        "replay was rejected by the lower runtime policy"
                    }
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::UnavailableTruthView => {
                        "truth view is unavailable for the requested historical evaluation"
                    }
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::UnsupportedTruthViewSelector => {
                        "truth view selector is unsupported for historical evaluation"
                    }
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::SourceCapabilityMismatch => {
                        "source capability does not match the requested historical evaluation"
                    }
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::BranchMismatch => {
                        "branch selector does not match the lower-runtime historical basis"
                    }
                    worth_runtime_bridge::facade::TruthViewPolicyRejectionKind::UnresolvedPolicyConflict => {
                        "historical policy conflict was left unresolved by the lower runtime"
                    }
                },
            },
        ),
    }
}

fn lower_admitted_policy(
    declaration: &HistoricalEvaluationDeclaration,
    policy: &ResolvedTruthViewPolicy,
    source_record: Option<&SourceMaterializationRecord>,
    requested_path_class: &RequestedHistoricalPathClass,
) -> HistoricalCapabilityDescriptor {
    let retention_available = matches!(
        policy.retention_admission(),
        TruthViewRetentionAdmission::SnapshotResident
    );
    let replay_permitted = declaration.replay_mode()
        != worth_runtime_bridge::facade::BridgeReplayMode::Disabled
        && matches!(
            policy.replay_continuity(),
            TruthViewReplayContinuity::ReplayPermitted | TruthViewReplayContinuity::ReplayRequired
        );
    let replay_required = declaration.replay_mode()
        == worth_runtime_bridge::facade::BridgeReplayMode::Required
        && matches!(
            policy.replay_continuity(),
            TruthViewReplayContinuity::ReplayRequired
        );
    let historical_lookup_available = matches!(
        policy.source_capability(),
        TruthViewSourceCapability::HistoricalLookupAndSnapshotRead
    );

    let admitted_path_class = match requested_path_class {
        RequestedHistoricalPathClass::RequestedRetainedSnapshotPath if retention_available => {
            Some(AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath)
        }
        RequestedHistoricalPathClass::RequestedDeltaReplayPath if replay_permitted => {
            Some(AdmittedHistoricalPathClass::AdmittedDeltaReplayPath)
        }
        RequestedHistoricalPathClass::RequestedFullReconstructionPath
            if declaration.selector().view_kind() == BridgeTruthViewKind::BranchHead
                && (replay_required || historical_lookup_available) =>
        {
            Some(AdmittedHistoricalPathClass::AdmittedFullReconstructionPath)
        }
        _ => None,
    };

    HistoricalCapabilityDescriptor::new(
        bridge_historical_basis(
            declaration
                .declaration_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting(),
        ),
        admitted_path_class,
        replay_permitted,
        replay_required,
        retention_available,
        historical_lookup_available,
        lower_reuse_descriptor(source_record),
    )
}

pub(crate) fn lower_materialization_from_decision_log(
    decision_log: &BridgeHistoricalEvaluationDecisionLog,
) -> Result<HistoricalMaterializationDescriptor, HistoricalEvaluationError> {
    lower_materialization_descriptor(
        bridge_historical_basis(
            decision_log
                .declaration_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting(),
        ),
        decision_log.materialization_path(),
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lower_materialization_from_artifact(
    artifact: &LoweredHistoricalEvaluationArtifact,
    requested_path_class: &RequestedHistoricalPathClass,
) -> Result<HistoricalMaterializationDescriptor, HistoricalEvaluationError> {
    let _ = requested_path_class;
    lower_materialization_descriptor(
        bridge_historical_basis(
            artifact
                .declaration_identity()
                .bridge_admission_evidence()
                .terminal_projection_for_reporting(),
        ),
        artifact.materialization_path(),
    )
}

fn lower_materialization_descriptor(
    basis: ResolvedBasisProof,
    path: BridgeHistoricalMaterializationPath,
) -> Result<HistoricalMaterializationDescriptor, HistoricalEvaluationError> {
    let resolved_path_class = resolved_path_class_for(path)?;
    let (actual_replay_span, actual_reconstruction_scope) = realized_work_for(path);
    Ok(
        HistoricalMaterializationDescriptor::new(basis, resolved_path_class)
            .with_realized_work(actual_replay_span, actual_reconstruction_scope),
    )
}

pub(crate) fn bridge_historical_basis(reporting_identity: &str) -> ResolvedBasisProof {
    let identity = worth_query_evidence_identity(WorthQueryEvidenceScope::ResolvedSnapshotBasis)
        .field_value(
            WorthQueryEvidenceTag::new("bridge_historical_declaration"),
            reporting_identity,
        )
        .seal();
    ResolvedBasisProof::new(identity, BasisResolutionMode::RuntimeDirect)
        .with_reporting_label(reporting_identity)
}

fn resolved_path_class_for(
    path: BridgeHistoricalMaterializationPath,
) -> Result<ResolvedHistoricalPathClass, HistoricalEvaluationError> {
    resolved_path_class_for_with_request(
        path,
        &RequestedHistoricalPathClass::RequestedRetainedSnapshotPath,
    )
}

fn resolved_path_class_for_with_request(
    path: BridgeHistoricalMaterializationPath,
    _requested_path_class: &RequestedHistoricalPathClass,
) -> Result<ResolvedHistoricalPathClass, HistoricalEvaluationError> {
    match path {
        BridgeHistoricalMaterializationPath::DirectSnapshotRead => {
            Ok(ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath)
        }
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot => {
            Ok(ResolvedHistoricalPathClass::ResolvedDeltaReplayPath)
        }
        BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot => {
            Ok(ResolvedHistoricalPathClass::ResolvedFullReconstructionPath)
        }
    }
}

fn realized_work_for(path: BridgeHistoricalMaterializationPath) -> (usize, usize) {
    match path {
        BridgeHistoricalMaterializationPath::DirectSnapshotRead => (0, 0),
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot => (1, 0),
        BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot => (0, 1),
    }
}

fn lower_reuse_descriptor(
    source_record: Option<&SourceMaterializationRecord>,
) -> HistoricalPathReuseDescriptor {
    let Some(record) = source_record else {
        return HistoricalPathReuseDescriptor::no_reuse();
    };

    if record.materialization_paths().iter().any(|path| {
        matches!(
            path,
            BridgeHistoricalMaterializationPath::DirectSnapshotRead
        )
    }) {
        HistoricalPathReuseDescriptor::retained_reuse()
    } else if record.materialization_paths().iter().any(|path| {
        matches!(
            path,
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
        )
    }) {
        HistoricalPathReuseDescriptor::with_replay_tail_reuse()
    } else {
        HistoricalPathReuseDescriptor::no_reuse()
    }
}
