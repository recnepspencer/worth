use super::*;

pub(super) enum HarnessTarget {
    CommittedRoute {
        commit_identity: TruthCommitIdentity,
    },
    Stream(super::stream::StreamHarnessTarget),
    Source(super::source::SourceHarnessTarget),
    Merge(super::merge::MergeHarnessTarget),
    Policy(super::policy::PolicyHarnessTarget),
    Speculation(super::speculation::SpeculationHarnessTarget),
    Structural(super::structural::StructuralHarnessTarget),
    Writeback(super::writeback::WritebackHarnessTarget),
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
    Stream(super::stream::StreamHarnessExecution),
    Source(super::source::SourceHarnessExecution),
    Merge(super::merge::MergeHarnessExecution),
    Policy(super::policy::PolicyHarnessExecution),
    Speculation(super::speculation::SpeculationHarnessExecution),
    Structural(super::structural::StructuralHarnessExecution),
    Writeback(super::writeback::WritebackHarnessExecution),
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
        BridgeHarnessTargetId::CommittedRoute { commit_identity } => Ok(HarnessTarget::CommittedRoute {
            commit_identity: commit_identity.clone(),
        }),
        BridgeHarnessTargetId::StreamRouting { commit_window } => {
            stream_window_target(commit_window).map(|window| {
                HarnessTarget::Stream(super::stream::StreamHarnessTarget::RoutingWindow { window })
            })
        }
        BridgeHarnessTargetId::StreamReplayAudit { commit_window } => {
            stream_window_target(commit_window).map(|window| {
                HarnessTarget::Stream(super::stream::StreamHarnessTarget::ReplayAuditWindow {
                    window,
                })
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
        BridgeHarnessTargetId::MergeExecute {
            declaration_identity,
        } => Ok(HarnessTarget::Merge(
            super::merge::MergeHarnessTarget::Execute {
                declaration_identity: declaration_identity.clone(),
            },
        )),
        BridgeHarnessTargetId::MergeReplay {
            declaration_identity,
        } => Ok(HarnessTarget::Merge(
            super::merge::MergeHarnessTarget::Replay {
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
        BridgeHarnessTargetId::StructuralRemapExact {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::RemapExact),
        BridgeHarnessTargetId::StructuralRemapAmbiguous {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::RemapAmbiguous),
        BridgeHarnessTargetId::StructuralRemapNoSafeMatch {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::RemapNoSafeMatch),
        BridgeHarnessTargetId::StructuralRemapLineageDivergence {
            declaration_identity,
        } => structural_target(
            declaration_identity,
            StructuralTargetKind::RemapLineageDivergence,
        ),
        BridgeHarnessTargetId::StructuralRemapIdentityConflict {
            declaration_identity,
        } => structural_target(
            declaration_identity,
            StructuralTargetKind::RemapIdentityConflict,
        ),
        BridgeHarnessTargetId::StructuralRemapReplay {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::RemapReplay),
        BridgeHarnessTargetId::StructuralBranchCompare {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::BranchCompare),
        BridgeHarnessTargetId::StructuralBranchReplay {
            declaration_identity,
        } => structural_target(declaration_identity, StructuralTargetKind::BranchReplay),
        BridgeHarnessTargetId::WritebackDuplicateCertification => Ok(HarnessTarget::Writeback(
            super::writeback::WritebackHarnessTarget::DuplicateCertification,
        )),
        BridgeHarnessTargetId::WritebackAuthorityDenialCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::AuthorityDenialCertification,
            ))
        }
        BridgeHarnessTargetId::WritebackFeedbackLoopCertification => Ok(HarnessTarget::Writeback(
            super::writeback::WritebackHarnessTarget::FeedbackLoopCertification,
        )),
        BridgeHarnessTargetId::WritebackReplayMismatchCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::ReplayMismatchCertification,
            ))
        }
        BridgeHarnessTargetId::WritebackExtensibleFamilyCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::ExtensibleFamilyCertification,
            ))
        }
        BridgeHarnessTargetId::WritebackMultiFamilyAdmissionBoundaryCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::MultiFamilyAdmissionBoundaryCertification,
            ))
        }
        BridgeHarnessTargetId::WritebackCrossFamilyReplayLoopIsolationCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::CrossFamilyReplayLoopIsolationCertification,
            ))
        }
        BridgeHarnessTargetId::WritebackHostMapperParityCertification => {
            Ok(HarnessTarget::Writeback(
                super::writeback::WritebackHarnessTarget::HostMapperParityCertification,
            ))
        }
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

fn stream_window_target(
    commit_window: &[TruthCommitIdentity],
) -> Result<super::stream::NativeStreamCommitWindow, BridgeHarnessError> {
    super::stream::NativeStreamCommitWindow::from_commits(commit_window.iter().cloned())
}

enum StructuralTargetKind {
    RemapExact,
    RemapAmbiguous,
    RemapNoSafeMatch,
    RemapLineageDivergence,
    RemapIdentityConflict,
    RemapReplay,
    BranchCompare,
    BranchReplay,
}

fn structural_target(
    declaration_identity: &crate::structural::StructuralIdentityDeclarationIdentity,
    kind: StructuralTargetKind,
) -> Result<HarnessTarget, BridgeHarnessError> {
    let declaration_identity = declaration_identity.clone();
    let target = match kind {
        StructuralTargetKind::RemapExact => {
            super::structural::StructuralHarnessTarget::RemapExact {
                declaration_identity,
            }
        }
        StructuralTargetKind::RemapAmbiguous => {
            super::structural::StructuralHarnessTarget::RemapAmbiguous {
                declaration_identity,
            }
        }
        StructuralTargetKind::RemapNoSafeMatch => {
            super::structural::StructuralHarnessTarget::RemapNoSafeMatch {
                declaration_identity,
            }
        }
        StructuralTargetKind::RemapLineageDivergence => {
            super::structural::StructuralHarnessTarget::RemapLineageDivergence {
                declaration_identity,
            }
        }
        StructuralTargetKind::RemapIdentityConflict => {
            super::structural::StructuralHarnessTarget::RemapIdentityConflict {
                declaration_identity,
            }
        }
        StructuralTargetKind::RemapReplay => {
            super::structural::StructuralHarnessTarget::RemapReplay {
                declaration_identity,
            }
        }
        StructuralTargetKind::BranchCompare => {
            super::structural::StructuralHarnessTarget::BranchCompare {
                declaration_identity,
            }
        }
        StructuralTargetKind::BranchReplay => {
            super::structural::StructuralHarnessTarget::BranchReplay {
                declaration_identity,
            }
        }
    };
    Ok(HarnessTarget::Structural(target))
}
