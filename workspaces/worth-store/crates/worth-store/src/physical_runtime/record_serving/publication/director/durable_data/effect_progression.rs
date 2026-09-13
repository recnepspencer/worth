use worth_store_physical_backend::QualifiedFilesystemMedia;

use super::super::RecordPublicationDirector;
use super::failure_outcome::{
    classify_dispatch_failure, map_canonical_failure, map_writeback_failure, pressure_basis,
    DispatchFailure,
};
use crate::physical_runtime::record_serving::residency::candidate_frame_residency::{
    CandidateFrame, CandidateFrameCoordinate, CandidateFrameWriteCompletion,
    StoreCandidateFramePublicationSession,
};
use crate::physical_runtime::record_serving::residency::publication_artifacts::PublicationRecordArtifacts;
use crate::physical_runtime::{
    DataDispatchedPhysicalMutation, IndeterminatePhysicalDataDispatch,
    PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome, PhysicalDataEffectSettlement,
    PhysicalRecordPressureBasis, RecordPublicationStage, WalDurablePhysicalMutation,
};

pub(super) struct DurableFrameDispatch<'director, 'media> {
    director: &'director RecordPublicationDirector,
    store_basis: PhysicalRecordPressureBasis,
    media: &'media QualifiedFilesystemMedia,
}

impl<'director, 'media> DurableFrameDispatch<'director, 'media> {
    pub(super) fn new(
        director: &'director RecordPublicationDirector,
        store_basis: PhysicalRecordPressureBasis,
        media: &'media QualifiedFilesystemMedia,
    ) -> Self {
        Self {
            director,
            store_basis,
            media,
        }
    }

    pub(super) fn execute(
        self,
        durable: WalDurablePhysicalMutation,
        mut residency: StoreCandidateFramePublicationSession<'_>,
    ) -> PhysicalDataDispatchOutcome {
        let artifacts = PublicationRecordArtifacts::new(&self.director.mutation);
        let mut effects = Vec::with_capacity(durable.data_frames().len());
        let mut completed_writebacks = 0_u64;
        for frame in durable.data_frames() {
            let effect = match self.dispatch_frame(
                &artifacts,
                &mut residency,
                frame,
                &mut completed_writebacks,
            ) {
                Ok(effect) => effect,
                Err(failure) => {
                    drop(residency);
                    return classify_dispatch_failure(
                        durable,
                        effects,
                        failure,
                        self.media,
                        self.director.generation,
                    );
                }
            };
            effects.push(effect);
        }
        if let Err(violation) = residency.require_complete() {
            return PhysicalDataDispatchOutcome::Indeterminate(
                IndeterminatePhysicalDataDispatch::new(
                    durable,
                    effects,
                    PhysicalDataDispatchFailureCause::CandidateFrameContract(violation),
                ),
            );
        }
        PhysicalDataDispatchOutcome::Dispatched(DataDispatchedPhysicalMutation::new(
            durable, effects,
        ))
    }

    fn dispatch_frame(
        &self,
        artifacts: &PublicationRecordArtifacts<'_>,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        frame: &crate::physical_runtime::durability::WalBoundPhysicalDataFrame,
        completed_writebacks: &mut u64,
    ) -> Result<PhysicalDataEffectSettlement, DispatchFailure> {
        let basis = frame.basis().clone();
        let target = basis.target();
        let coordinate = target.coordinate();
        let candidate = CandidateFrame::new(
            super::candidate_role(target.kind()),
            CandidateFrameCoordinate::new(coordinate.artifact(), coordinate.offset()),
            frame.bytes().to_vec(),
        );
        let pressure_basis = pressure_basis(
            self.director.durability.store_identity(),
            candidate.coordinate(),
            coordinate.length(),
        )
        .unwrap_or(self.store_basis);
        let existing_artifact = coordinate.offset() != 0;
        let completion = {
            let mut after_admission_before_effect = || {
                // The first existing-artifact writeback is the backend gate's
                // target. Pause the next admitted claim so the backend gate
                // can be released before the competing mutation is admitted.
                if existing_artifact && *completed_writebacks != 0 {
                    self.director.mutations.reach_checkpoint(
                        crate::physical_runtime::durability::PhysicalMutationCheckpoint::
                            AfterWritebackAdmissionBeforeEffect,
                    );
                }
            };
            self.write_candidate(
                artifacts,
                residency,
                candidate,
                coordinate,
                pressure_basis,
                &mut after_admission_before_effect,
            )?
        };
        if existing_artifact {
            *completed_writebacks = completed_writebacks.saturating_add(1);
        }
        let effect = completion.effect().ok_or({
            DispatchFailure::Uncertain(PhysicalDataDispatchFailureCause::MissingEffectSettlement)
        })?;
        Ok(PhysicalDataEffectSettlement::from_candidate(basis, effect))
    }

    fn write_candidate(
        &self,
        artifacts: &PublicationRecordArtifacts<'_>,
        residency: &mut StoreCandidateFramePublicationSession<'_>,
        candidate: CandidateFrame,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
        pressure_basis: PhysicalRecordPressureBasis,
        after_admission_before_effect: &mut dyn FnMut(),
    ) -> Result<CandidateFrameWriteCompletion, DispatchFailure> {
        if coordinate.offset() == 0 {
            artifacts
                .write_new_candidate(
                    RecordPublicationStage::CandidateDataWrite,
                    residency,
                    candidate,
                    coordinate.artifact(),
                )
                .map_err(|failure| {
                    map_canonical_failure(failure, self.director.generation, pressure_basis)
                })
        } else {
            artifacts
                .write_existing_artifact_candidate_with_pause(
                    residency,
                    candidate,
                    self.director.residency.writeback(),
                    after_admission_before_effect,
                )
                .map_err(|failure| {
                    map_writeback_failure(failure, self.director.generation, pressure_basis)
                })
        }
    }
}
