use sha2::{Digest, Sha256};
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::RecordFrameCoordinate;

use super::{
    CandidateFrame, CandidateFrameFailurePosture, CandidateFrameWriteCompletion,
    CandidateFrameWriteFailure, StoreCandidateFramePublicationSession,
};
use crate::physical_runtime::record_serving::{
    residency::{
        dirty::{
            AdmittedDirtyFrame, PhysicalRecordWritebackFailureCause,
            PhysicalRecordWritebackFailureEvidence, PhysicalWritebackExecution,
            PhysicalWritebackTransitionFailure,
        },
        FrameWritebackPort,
    },
    RecordAppendDenial,
};

impl StoreCandidateFramePublicationSession<'_> {
    pub(in crate::physical_runtime::record_serving) fn write_frame_via_writeback_with_pause(
        &mut self,
        frame: CandidateFrame,
        writeback: &FrameWritebackPort,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<
        (
            CandidateFrameWriteCompletion,
            super::super::dirty::PhysicalWritebackSettlement,
        ),
        CandidateFrameWriteFailure<PhysicalRecordWritebackFailureEvidence>,
    > {
        let (resident, expectation, frame) = self
            .retain_submitted_frame(frame)
            .map_err(super::RecoverableCandidateFrameWriteFailure::into_cause)?;
        let coordinate = RecordFrameCoordinate::new(
            resident.coordinate().artifact(),
            resident.coordinate().offset(),
            u32::try_from(resident.bytes().len())
                .expect("admitted candidate frame lengths are u32-bounded"),
        )
        .expect("admitted candidate frames are nonempty and offset-bounded");
        let payload_digest: [u8; 32] = Sha256::digest(resident.bytes()).into();
        let dirty = AdmittedDirtyFrame::candidate(
            coordinate,
            resident
                .into_dirty()
                .map_err(|denial| CandidateFrameWriteFailure::Residency {
                    denial,
                    posture: CandidateFrameFailurePosture::UnsettledBeforeEffect,
                })?,
        );
        let prepared = match writeback.prepare(
            dirty,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ) {
            Ok(prepared) => prepared,
            Err(failure) => return discard_transition_failure(None, failure),
        };
        let identity = prepared.identity();
        let ready = match writeback.request_ready(prepared) {
            Ok(ready) => ready,
            Err(failure) => return discard_transition_failure(Some(identity), failure),
        };
        let admitted = match writeback.admit(ready, None) {
            Ok(admitted) => admitted,
            Err(failure) => return discard_transition_failure(Some(identity), failure),
        };
        after_admission_before_effect();
        let execution = match writeback.execute(admitted) {
            Ok(execution) => execution,
            Err(failure) => return discard_transition_failure(Some(identity), failure),
        };
        match execution {
            PhysicalWritebackExecution::Clean(settlement) => {
                let completion = CandidateFrameWriteCompletion::writeback(
                    expectation.frame_bytes(),
                    coordinate,
                    payload_digest,
                    settlement,
                );
                self.complete_frame(expectation, &completion, frame)
                    .map_err(super::RecoverableCandidateFrameWriteFailure::into_cause)?;
                Ok((completion, settlement))
            }
            PhysicalWritebackExecution::Retryable(retryable) => {
                let settlement = retryable.settlement();
                retryable
                    .into_dirty()
                    .discard()
                    .map_err(unsettled_residency_failure)?;
                Err(CandidateFrameWriteFailure::Effect(
                    PhysicalRecordWritebackFailureEvidence::settled(
                        PhysicalRecordWritebackFailureCause::RetryableNoEffect,
                        settlement,
                    ),
                ))
            }
            PhysicalWritebackExecution::InspectionRequired(inspection) => {
                Err(CandidateFrameWriteFailure::Effect(
                    PhysicalRecordWritebackFailureEvidence::settled(
                        PhysicalRecordWritebackFailureCause::InspectionRequired,
                        inspection.settlement(),
                    ),
                ))
            }
        }
    }
}

fn discard_transition_failure(
    identity: Option<crate::physical_runtime::PhysicalWorkIdentity>,
    failure: PhysicalWritebackTransitionFailure,
) -> Result<
    (
        CandidateFrameWriteCompletion,
        super::super::dirty::PhysicalWritebackSettlement,
    ),
    CandidateFrameWriteFailure<PhysicalRecordWritebackFailureEvidence>,
> {
    let cause = failure.cause();
    let dirty = failure.into_dirty();
    let coordinate = dirty.coordinate();
    dirty.discard().map_err(unsettled_residency_failure)?;
    Err(CandidateFrameWriteFailure::Effect(
        PhysicalRecordWritebackFailureEvidence::transition(identity, cause, coordinate),
    ))
}

fn unsettled_residency_failure(
    reason: worth_store_buffer_pool::PhysicalResidencyDenial,
) -> CandidateFrameWriteFailure<PhysicalRecordWritebackFailureEvidence> {
    CandidateFrameWriteFailure::Residency {
        denial: RecordAppendDenial::from_residency(reason),
        posture: CandidateFrameFailurePosture::UnsettledBeforeEffect,
    }
}
