use worth_store_physical_format::RecordFrameCoordinate;

use super::candidate_frame_residency::{
    CandidateFrame, CandidateFrameCoordinate, CandidateFramePhysicalWrite,
    CandidateFrameWriteCompletion, CandidateFrameWriteFailure,
    RecoverableCandidateFrameWriteFailure, StoreCandidateFramePublicationSession,
};

pub(in crate::physical_runtime::record_serving) struct PublicationRecordArtifacts<'port> {
    mutation: &'port super::super::CanonicalRecordMutationPort,
    retry: Option<&'port super::super::RetirementCandidateRetryScope>,
}

impl<'port> PublicationRecordArtifacts<'port> {
    pub(in crate::physical_runtime::record_serving) fn new(
        mutation: &'port super::super::CanonicalRecordMutationPort,
    ) -> Self {
        Self {
            mutation,
            retry: None,
        }
    }
    pub(in crate::physical_runtime::record_serving) fn retirement_retry(
        mutation: &'port super::super::CanonicalRecordMutationPort,
        retry: &'port super::super::RetirementCandidateRetryScope,
    ) -> Self {
        Self {
            mutation,
            retry: Some(retry),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn write_new_candidate(
        &self,
        stage: super::super::RecordPublicationStage,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<
        CandidateFrameWriteCompletion,
        CandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        let coordinate = frame.coordinate();
        residency.write_frame(frame, &mut |bytes| {
            self.write_new_frame_with_pause(stage, coordinate, bytes, after_admission_before_effect)
        })
    }

    /// Writes one WAL-authorized blob chunk or tree-node arena frame through
    /// the Store's bounded ingest-pressure attempt.
    pub(in crate::physical_runtime::record_serving) fn write_blob_ingest_candidate(
        &self,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<
        CandidateFrameWriteCompletion,
        CandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        let target = frame.coordinate();
        residency.write_frame(frame, &mut |bytes| {
            let length = u32::try_from(bytes.len()).expect("candidate frame length is bounded");
            let coordinate = RecordFrameCoordinate::new(target.artifact(), target.offset(), length)
                .expect("candidate frames are nonempty and offset-bounded");
            let prepared = self.mutation.prepare_blob_ingest_artifact(
                super::super::RecordPublicationStage::CandidateDataWrite,
                coordinate,
                bytes,
            )?;
            after_admission_before_effect();
            Ok(prepared.execute()?.into_physical())
        })
    }

    /// Spend reclaim background capacity on the actual WAL-authorized
    /// control-frame write, including publication-only drop batches.
    pub(in crate::physical_runtime::record_serving) fn write_blob_reclaim_candidate(
        &self,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<
        CandidateFrameWriteCompletion,
        CandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        let target = frame.coordinate();
        residency.write_frame(frame, &mut |bytes| {
            let length = u32::try_from(bytes.len()).expect("candidate frame length is bounded");
            let coordinate = RecordFrameCoordinate::new(target.artifact(), target.offset(), length)
                .expect("candidate frames are nonempty and offset-bounded");
            let prepared = self.mutation.prepare_blob_reclaim_artifact(
                super::super::RecordPublicationStage::CandidateDataWrite,
                coordinate,
                bytes,
            )?;
            after_admission_before_effect();
            Ok(prepared.execute()?.into_physical())
        })
    }

    pub(in crate::physical_runtime::record_serving) fn write_new_candidate_recoverable(
        &self,
        stage: super::super::RecordPublicationStage,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: CandidateFrame,
    ) -> Result<
        CandidateFrameWriteCompletion,
        RecoverableCandidateFrameWriteFailure<super::super::CanonicalRecordMutationFailure>,
    > {
        let coordinate = frame.coordinate();
        residency.write_frame_recoverable(frame, &mut |bytes| {
            self.write_new_frame(stage, coordinate, bytes)
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
        target: CandidateFrameCoordinate,
        bytes: &[u8],
    ) -> Result<CandidateFramePhysicalWrite, super::super::CanonicalRecordMutationFailure> {
        self.write_new_frame_with_pause(stage, target, bytes, &mut || {})
    }

    fn write_new_frame_with_pause(
        &self,
        stage: super::super::RecordPublicationStage,
        target: CandidateFrameCoordinate,
        bytes: &[u8],
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<CandidateFramePhysicalWrite, super::super::CanonicalRecordMutationFailure> {
        let length = u32::try_from(bytes.len()).expect("candidate frame length is u32-bounded");
        let coordinate = RecordFrameCoordinate::new(target.artifact(), target.offset(), length)
            .expect("candidate frames are nonempty and offset-bounded");
        if let Some(retry) = self.retry {
            let prepared = self.mutation.prepare_retirement_candidate_retry(
                stage,
                coordinate,
                bytes,
                retry.clone(),
            )?;
            after_admission_before_effect();
            return Ok(prepared.execute()?.into_physical());
        }
        let prepared = self
            .mutation
            .prepare_new_artifact(stage, coordinate, bytes)?;
        after_admission_before_effect();
        Ok(prepared.execute()?.into_physical())
    }
}
