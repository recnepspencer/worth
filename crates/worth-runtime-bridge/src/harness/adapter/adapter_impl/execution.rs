use super::*;

pub(super) enum HarnessTarget {
    CommittedRoute {
        commit_identity: TruthCommitIdentity,
    },
    Source(super::source::SourceHarnessTarget),
    Policy(super::policy::PolicyHarnessTarget),
    Speculation(super::speculation::SpeculationHarnessTarget),
    HistoricalCommit {
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    },
    BranchHead {
        branch_identity: TruthBranchIdentity,
    },
}

pub(super) enum HarnessExecution {
    Route {
        result: crate::facade::BridgeRouteResult,
        continuity_summary: Option<(
            crate::facade::BridgeContinuityArtifact,
            crate::facade::BridgeCanonicalContinuityRecord,
        )>,
    },
    Historical {
        artifact: crate::facade::LoweredHistoricalEvaluationArtifact,
        record: crate::facade::BridgeCanonicalHistoricalEvaluationRecord,
        explanation: BridgeHistoricalEvaluationExplanation,
    },
    Source(super::source::SourceHarnessExecution),
    Policy(super::policy::PolicyHarnessExecution),
    Speculation(super::speculation::SpeculationHarnessExecution),
}

pub(super) fn execute_historical_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    declaration: HistoricalEvaluationDeclaration,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<HarnessExecution, BridgeHarnessError> {
    let planned = runtime_bridge
        .plan_truth_view_packet(declaration, SnapshotReadPacket::new(vec![]), execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge historical planning failed: {error}"))
        })?;
    let observation = runtime_bridge
        .materialize_truth_view_observation(planned, execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge historical materialization failed: {error}"))
        })?;
    let artifact = runtime_bridge.lower_historical_evaluation_artifact(&observation);
    let record = runtime_bridge.canonicalize_historical_evaluation_record(&observation);
    let explanation = runtime_bridge
        .diagnostics()
        .explain_historical_evaluation_record(&record);

    Ok(HarnessExecution::Historical {
        artifact,
        record,
        explanation,
    })
}

pub(super) fn harness_target_from_id(
    target: &BridgeHarnessTargetId,
) -> Result<HarnessTarget, BridgeHarnessError> {
    match target {
        BridgeHarnessTargetId::CommittedRoute { commit_identity } => {
            Ok(HarnessTarget::CommittedRoute {
                commit_identity: commit_identity.clone(),
            })
        }
        BridgeHarnessTargetId::SourceMaterialize {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::Materialize {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::SourceMaterializeBatch {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::MaterializeBatch {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::SourceReplay {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::Replay {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::SourceRejectUnregistered {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::RejectUnregistered {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::SourceRejectOpenSnapshot {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::RejectOpenSnapshot {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::SourceRejectSnapshotDrift {
            declaration_identity,
        } => Ok(HarnessTarget::Source(
            super::source::SourceHarnessTarget::RejectSnapshotDrift {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::PolicyProvenanceCertification => Ok(HarnessTarget::Policy(
            super::policy::PolicyHarnessTarget::ProvenanceCertification,
        )),
        BridgeHarnessTargetId::PolicyRejectionCertification => Ok(HarnessTarget::Policy(
            super::policy::PolicyHarnessTarget::RejectionCertification,
        )),
        BridgeHarnessTargetId::PolicyAmbientLeakCertification => Ok(HarnessTarget::Policy(
            super::policy::PolicyHarnessTarget::AmbientLeakCertification,
        )),
        BridgeHarnessTargetId::SpeculationDiscardCertification => Ok(HarnessTarget::Speculation(
            super::speculation::SpeculationHarnessTarget::DiscardCertification,
        )),
        BridgeHarnessTargetId::SpeculationPromotionCertification => Ok(HarnessTarget::Speculation(
            super::speculation::SpeculationHarnessTarget::PromotionCertification,
        )),
        BridgeHarnessTargetId::SpeculationChurnCertification => Ok(HarnessTarget::Speculation(
            super::speculation::SpeculationHarnessTarget::ChurnCertification,
        )),
        BridgeHarnessTargetId::HistoricalCommit {
            branch_identity,
            commit_identity,
        } => Ok(HarnessTarget::HistoricalCommit {
            branch_identity: branch_identity.clone(),
            commit_identity: commit_identity.clone(),
        }),
        BridgeHarnessTargetId::BranchHead { branch_identity } => Ok(HarnessTarget::BranchHead {
            branch_identity: branch_identity.clone(),
        }),
    }
}
