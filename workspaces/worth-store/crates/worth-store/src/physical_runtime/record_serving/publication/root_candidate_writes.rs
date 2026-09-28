use worth_store_physical_format::RecordArtifactFile;

use super::{PublicationPlan, RecordPublicationStage};
use crate::physical_runtime::record_serving::residency::{
    frame_ports::{
        CandidateFrame, CandidateFrameCoordinate, CandidateFrameFailurePosture, CandidateFrameRole,
        CandidateFrameWriteFailure, RecoverableCandidateFrameWriteFailure,
        StoreCandidateFramePublicationSession,
    },
    publication_artifacts::PublicationRecordArtifacts,
};

type RootCandidateFrameWriteFailure =
    RecoverableCandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>;
type RootCandidateFrameWriteFailureCause =
    CandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>;

pub(in crate::physical_runtime::record_serving) struct WrittenRootCandidateArtifacts {
    pub(in crate::physical_runtime::record_serving) plan: PublicationPlan,
    pub(in crate::physical_runtime::record_serving) artifacts: Box<[RecordArtifactFile]>,
}

pub(in crate::physical_runtime::record_serving) enum RootCandidateWriteFailure {
    RetryableNoEffect {
        plan: PublicationPlan,
        failed_artifact: RecordArtifactFile,
        cause: RootCandidateWriteFailureKind,
    },
    InspectionRequired {
        plan: PublicationPlan,
        completed_artifacts: Box<[RecordArtifactFile]>,
        failed_artifact: RecordArtifactFile,
        cause: RootCandidateWriteFailureKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RootCandidateWriteFailureKind {
    Contract {
        violation: crate::physical_runtime::CandidateFrameContractViolation,
        posture: RootCandidateWriteFailurePosture,
    },
    Effect {
        fate: crate::physical_runtime::PhysicalWorkEffectFate,
    },
    Residency {
        denial: crate::physical_runtime::RecordAppendDenial,
        posture: RootCandidateWriteFailurePosture,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RootCandidateWriteFailurePosture {
    ProvenNoEffect,
    UnsettledBeforeEffect,
    EffectPossible,
}

fn project_failure_kind(
    cause: &RootCandidateFrameWriteFailureCause,
) -> RootCandidateWriteFailureKind {
    match cause {
        CandidateFrameWriteFailure::Contract { violation, posture } => {
            RootCandidateWriteFailureKind::Contract {
                violation: *violation,
                posture: project_posture(*posture),
            }
        }
        CandidateFrameWriteFailure::Effect(failure) => RootCandidateWriteFailureKind::Effect {
            fate: failure.effect_fate(),
        },
        CandidateFrameWriteFailure::Residency { denial, posture } => {
            RootCandidateWriteFailureKind::Residency {
                denial: denial.clone(),
                posture: project_posture(*posture),
            }
        }
    }
}

fn proves_no_effect(cause: &RootCandidateFrameWriteFailureCause) -> bool {
    match cause {
        CandidateFrameWriteFailure::Contract { posture, .. }
        | CandidateFrameWriteFailure::Residency { posture, .. } => {
            *posture == CandidateFrameFailurePosture::ProvenNoEffect
        }
        CandidateFrameWriteFailure::Effect(failure) => {
            failure.effect_fate() == crate::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
        }
    }
}

const fn project_posture(
    posture: CandidateFrameFailurePosture,
) -> RootCandidateWriteFailurePosture {
    match posture {
        CandidateFrameFailurePosture::ProvenNoEffect => {
            RootCandidateWriteFailurePosture::ProvenNoEffect
        }
        CandidateFrameFailurePosture::UnsettledBeforeEffect => {
            RootCandidateWriteFailurePosture::UnsettledBeforeEffect
        }
        CandidateFrameFailurePosture::EffectPossible => {
            RootCandidateWriteFailurePosture::EffectPossible
        }
    }
}

pub(in crate::physical_runtime::record_serving) fn write_root_candidate_artifacts(
    artifacts: &PublicationRecordArtifacts<'_>,
    mut plan: PublicationPlan,
    residency: &mut StoreCandidateFramePublicationSession<'_>,
) -> Result<WrittenRootCandidateArtifacts, RootCandidateWriteFailure> {
    let mut written = Vec::with_capacity(plan.manifests.len() + 2);
    for index in 0..plan.manifests.len() {
        let (target, bytes) = &mut plan.manifests[index];
        let coordinate = CandidateFrameCoordinate::new(target.artifact(), target.offset());
        let artifact = target.artifact();
        let role = if matches!(artifact, RecordArtifactFile::ExtentArena { .. }) {
            CandidateFrameRole::ExtentManifest
        } else {
            CandidateFrameRole::ManifestBlock
        };
        let bytes = std::mem::take(bytes);
        if let Err(cause) =
            write_candidate(artifacts, &mut plan, residency, coordinate, role, bytes)
        {
            return Err(write_failure(plan, written, coordinate, cause));
        }
        written.push(artifact);
    }
    let root = plan.root;
    let root_bytes = std::mem::take(&mut plan.root_bytes);
    if let Err(cause) = write_candidate(
        artifacts,
        &mut plan,
        residency,
        CandidateFrameCoordinate::new(root, 0),
        CandidateFrameRole::RootManifest,
        root_bytes,
    ) {
        return Err(write_failure(
            plan,
            written,
            CandidateFrameCoordinate::new(root, 0),
            cause,
        ));
    }
    written.push(root);
    let previous_selector = plan.previous_selector_candidate;
    let previous_selector_bytes = std::mem::take(&mut plan.previous_selector_bytes);
    if let Err(cause) = write_candidate(
        artifacts,
        &mut plan,
        residency,
        CandidateFrameCoordinate::new(previous_selector, 0),
        CandidateFrameRole::RootSelectorCandidate,
        previous_selector_bytes,
    ) {
        return Err(write_failure(
            plan,
            written,
            CandidateFrameCoordinate::new(previous_selector, 0),
            cause,
        ));
    }
    written.push(previous_selector);
    let current_selector = plan.current_selector_candidate;
    let current_selector_bytes = std::mem::take(&mut plan.current_selector_bytes);
    if let Err(cause) = write_candidate(
        artifacts,
        &mut plan,
        residency,
        CandidateFrameCoordinate::new(current_selector, 0),
        CandidateFrameRole::RootSelectorCandidate,
        current_selector_bytes,
    ) {
        return Err(write_failure(
            plan,
            written,
            CandidateFrameCoordinate::new(current_selector, 0),
            cause,
        ));
    }
    written.push(current_selector);
    let candidate = plan.candidate;
    let catalog_bytes = std::mem::take(&mut plan.catalog_bytes);
    if let Err(cause) = write_candidate(
        artifacts,
        &mut plan,
        residency,
        CandidateFrameCoordinate::new(candidate, 0),
        CandidateFrameRole::CatalogCandidate,
        catalog_bytes,
    ) {
        return Err(write_failure(
            plan,
            written,
            CandidateFrameCoordinate::new(candidate, 0),
            cause,
        ));
    }
    written.push(candidate);
    // Many manifest frames can inhabit one arena; synchronize the artifact
    // once after all its admitted frame effects have completed.
    written.sort_unstable();
    written.dedup();
    Ok(WrittenRootCandidateArtifacts {
        plan,
        artifacts: written.into_boxed_slice(),
    })
}

fn write_candidate(
    artifacts: &PublicationRecordArtifacts<'_>,
    plan: &mut PublicationPlan,
    residency: &mut StoreCandidateFramePublicationSession<'_>,
    coordinate: CandidateFrameCoordinate,
    role: CandidateFrameRole,
    bytes: Vec<u8>,
) -> Result<(), RootCandidateFrameWriteFailure> {
    let frame = CandidateFrame::new(role, coordinate, bytes);
    let completion = artifacts.write_new_candidate_recoverable(
        RecordPublicationStage::ManifestSynchronization,
        residency,
        frame,
    )?;
    plan.observation
        .observe_transfer(completion.frame_bytes() as usize);
    Ok(())
}

fn write_failure(
    mut plan: PublicationPlan,
    completed_artifacts: Vec<RecordArtifactFile>,
    failed_coordinate: CandidateFrameCoordinate,
    cause: RootCandidateFrameWriteFailure,
) -> RootCandidateWriteFailure {
    let (cause, failed_bytes) = cause.into_parts();
    restore_failed_bytes(&mut plan, failed_coordinate, failed_bytes);
    let failed_artifact = failed_coordinate.artifact();
    let kind = project_failure_kind(&cause);
    if completed_artifacts.is_empty() && proves_no_effect(&cause) {
        RootCandidateWriteFailure::RetryableNoEffect {
            plan,
            failed_artifact,
            cause: kind,
        }
    } else {
        RootCandidateWriteFailure::InspectionRequired {
            plan,
            completed_artifacts: completed_artifacts.into_boxed_slice(),
            failed_artifact,
            cause: kind,
        }
    }
}

fn restore_failed_bytes(
    plan: &mut PublicationPlan,
    coordinate: CandidateFrameCoordinate,
    bytes: Vec<u8>,
) {
    let artifact = coordinate.artifact();
    if let Some((_, target)) = plan.manifests.iter_mut().find(|(candidate, _)| {
        candidate.artifact() == artifact && candidate.offset() == coordinate.offset()
    }) {
        debug_assert!(target.is_empty());
        *target = bytes;
    } else if plan.root == artifact {
        debug_assert!(plan.root_bytes.is_empty());
        plan.root_bytes = bytes;
    } else if plan.previous_selector_candidate == artifact {
        debug_assert!(plan.previous_selector_bytes.is_empty());
        plan.previous_selector_bytes = bytes;
    } else if plan.current_selector_candidate == artifact {
        debug_assert!(plan.current_selector_bytes.is_empty());
        plan.current_selector_bytes = bytes;
    } else if plan.candidate == artifact {
        debug_assert!(plan.catalog_bytes.is_empty());
        plan.catalog_bytes = bytes;
    } else {
        unreachable!("failed root candidate artifact belongs to its publication plan")
    }
}
