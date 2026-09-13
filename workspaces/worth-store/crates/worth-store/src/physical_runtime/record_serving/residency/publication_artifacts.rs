use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::candidate_frame_residency::{
    CandidateFrame, CandidateFramePhysicalWrite, CandidateFrameWriteCompletion,
    CandidateFrameWriteFailure, RecoverableCandidateFrameWriteFailure,
    StoreCandidateFramePublicationSession,
};

pub(in crate::physical_runtime::record_serving) struct PublicationRecordArtifacts<'port> {
    mutation: &'port super::super::CanonicalRecordMutationPort,
}

impl<'port> PublicationRecordArtifacts<'port> {
    pub(in crate::physical_runtime::record_serving) fn new(
        mutation: &'port super::super::CanonicalRecordMutationPort,
    ) -> Self {
        Self { mutation }
    }

    pub(in crate::physical_runtime::record_serving) fn write_new_candidate(
        &self,
        stage: super::super::RecordPublicationStage,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        artifact: RecordArtifactFile,
    ) -> Result<
        CandidateFrameWriteCompletion,
        CandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        residency.write_frame(frame, &mut |bytes| {
            self.write_new_frame(stage, artifact, bytes)
        })
    }

    pub(in crate::physical_runtime::record_serving) fn write_new_candidate_recoverable(
        &self,
        stage: super::super::RecordPublicationStage,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        artifact: RecordArtifactFile,
    ) -> Result<
        CandidateFrameWriteCompletion,
        RecoverableCandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        residency.write_frame_recoverable(frame, &mut |bytes| {
            self.write_new_frame(stage, artifact, bytes)
        })
    }

    pub(in crate::physical_runtime::record_serving) fn write_existing_artifact_candidate_with_pause(
        &self,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        writeback: &super::FrameWritebackPort,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<
        CandidateFrameWriteCompletion,
        CandidateFrameWriteFailure<super::dirty::PhysicalRecordWritebackFailureEvidence>,
    > {
        residency
            .write_frame_via_writeback_with_pause(frame, writeback, after_admission_before_effect)
            .map(|(completion, _settlement)| completion)
    }

    fn write_new_frame(
        &self,
        stage: super::super::RecordPublicationStage,
        artifact: RecordArtifactFile,
        bytes: &[u8],
    ) -> Result<CandidateFramePhysicalWrite, super::super::CanonicalRecordMutationFailure> {
        let length = u32::try_from(bytes.len()).expect("candidate frame length is u32-bounded");
        let coordinate = RecordFrameCoordinate::new(artifact, 0, length)
            .expect("candidate frames are nonempty and offset-bounded");
        Ok(self
            .mutation
            .prepare_new_artifact(stage, coordinate, bytes)?
            .execute()?
            .into_physical())
    }
}
