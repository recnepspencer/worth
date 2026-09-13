use super::*;

fn contract<EffectFailure>(
    violation: CandidateFrameContractViolation,
    posture: CandidateFrameFailurePosture,
    frame: CandidateFrame,
) -> RecoverableCandidateFrameWriteFailure<EffectFailure> {
    RecoverableCandidateFrameWriteFailure::new(
        CandidateFrameWriteFailure::Contract { violation, posture },
        frame,
    )
}

fn residency<EffectFailure>(
    denial: RecordAppendDenial,
    posture: CandidateFrameFailurePosture,
    frame: CandidateFrame,
) -> RecoverableCandidateFrameWriteFailure<EffectFailure> {
    RecoverableCandidateFrameWriteFailure::new(
        CandidateFrameWriteFailure::Residency { denial, posture },
        frame,
    )
}

pub(in crate::physical_runtime::record_serving) trait CandidateFrameEffectFailure {
    fn effect_fate(&self) -> crate::physical_runtime::PhysicalWorkEffectFate;
}

impl CandidateFrameEffectFailure for super::super::super::CanonicalRecordMutationFailure {
    fn effect_fate(&self) -> crate::physical_runtime::PhysicalWorkEffectFate {
        self.effect_fate()
    }
}

impl StoreCandidateFramePublicationSession<'_> {
    pub(in crate::physical_runtime::record_serving) fn write_frame<EffectFailure>(
        &mut self,
        frame: CandidateFrame,
        store_write: &mut dyn FnMut(&[u8]) -> Result<CandidateFramePhysicalWrite, EffectFailure>,
    ) -> Result<CandidateFrameWriteCompletion, CandidateFrameWriteFailure<EffectFailure>>
    where
        EffectFailure: CandidateFrameEffectFailure,
    {
        self.write_frame_recoverable(frame, store_write)
            .map_err(RecoverableCandidateFrameWriteFailure::into_cause)
    }

    pub(in crate::physical_runtime::record_serving) fn write_frame_recoverable<EffectFailure>(
        &mut self,
        frame: CandidateFrame,
        store_write: &mut dyn FnMut(&[u8]) -> Result<CandidateFramePhysicalWrite, EffectFailure>,
    ) -> Result<CandidateFrameWriteCompletion, RecoverableCandidateFrameWriteFailure<EffectFailure>>
    where
        EffectFailure: CandidateFrameEffectFailure,
    {
        let (resident, expectation, frame) = self.retain_submitted_frame(frame)?;
        let physical = match store_write(resident.bytes()) {
            Ok(physical) => physical,
            Err(failure) => {
                if failure.effect_fate()
                    == crate::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
                {
                    if let Err(denial) = resident.discard() {
                        return Err(residency(
                            denial,
                            CandidateFrameFailurePosture::UnsettledBeforeEffect,
                            frame,
                        ));
                    }
                }
                return Err(RecoverableCandidateFrameWriteFailure::new(
                    CandidateFrameWriteFailure::Effect(failure),
                    frame,
                ));
            }
        };
        let settlement = match physical.settle_residency(
            resident.store_identity(),
            resident.coordinate(),
            resident.bytes(),
        ) {
            Ok(settlement) => settlement,
            Err(violation) => {
                return Err(contract(
                    violation,
                    CandidateFrameFailurePosture::EffectPossible,
                    frame,
                ))
            }
        };
        let completion = match resident.publish_clean(settlement) {
            Ok(completion) => completion,
            Err(denial) => {
                return Err(residency(
                    denial,
                    CandidateFrameFailurePosture::EffectPossible,
                    frame,
                ))
            }
        };
        self.complete_frame(expectation, &completion, frame)?;
        Ok(completion)
    }

    pub(super) fn retain_submitted_frame<EffectFailure>(
        &mut self,
        frame: CandidateFrame,
    ) -> Result<
        (
            Box<dyn ResidentCandidateFrame>,
            RetainedFrameExpectation,
            CandidateFrame,
        ),
        RecoverableCandidateFrameWriteFailure<EffectFailure>,
    > {
        let expected = match self.validate_submitted_frame(&frame) {
            Ok(expected) => expected,
            Err((violation, posture)) => return Err(contract(violation, posture, frame)),
        };
        let expectation = RetainedFrameExpectation::capture(expected, &frame);
        let next_bytes = self.resident_bytes.saturating_add(expectation.frame_bytes);
        if next_bytes > self.declaration.total_frame_bytes() {
            return Err(contract(
                CandidateFrameContractViolation::FrameBytesExceedDeclaration,
                CandidateFrameFailurePosture::ProvenNoEffect,
                frame,
            ));
        }
        let resident = match self.residency.retain(&frame) {
            Ok(resident) => resident,
            Err(denial) => {
                return Err(residency(
                    denial,
                    CandidateFrameFailurePosture::ProvenNoEffect,
                    frame,
                ))
            }
        };
        if let Err(violation) = verify_retained_frame(resident.as_ref(), expectation) {
            if let Err(denial) = resident.discard() {
                return Err(residency(
                    denial,
                    CandidateFrameFailurePosture::UnsettledBeforeEffect,
                    frame,
                ));
            }
            return Err(contract(
                violation,
                CandidateFrameFailurePosture::ProvenNoEffect,
                frame,
            ));
        }
        Ok((resident, expectation, frame))
    }

    pub(super) fn complete_frame<EffectFailure>(
        &mut self,
        expectation: RetainedFrameExpectation,
        completion: &CandidateFrameWriteCompletion,
        frame: CandidateFrame,
    ) -> Result<(), RecoverableCandidateFrameWriteFailure<EffectFailure>> {
        if completion.frame_bytes() != expectation.frame_bytes {
            return Err(contract(
                CandidateFrameContractViolation::FrameCompletionMismatch,
                CandidateFrameFailurePosture::EffectPossible,
                frame,
            ));
        }
        self.resident_frames = self.resident_frames.saturating_add(1);
        self.resident_bytes = self.resident_bytes.saturating_add(expectation.frame_bytes);
        self.next_declaration += 1;
        Ok(())
    }

    fn validate_submitted_frame(
        &self,
        frame: &CandidateFrame,
    ) -> Result<
        CandidateFrameDeclaration,
        (
            CandidateFrameContractViolation,
            CandidateFrameFailurePosture,
        ),
    > {
        if !coordinate_matches_role(frame.role(), frame.coordinate().artifact()) {
            return Err((
                CandidateFrameContractViolation::CoordinateRoleMismatch,
                CandidateFrameFailurePosture::ProvenNoEffect,
            ));
        }
        let Some(expected) = self.declaration.declaration(self.next_declaration) else {
            return Err((
                CandidateFrameContractViolation::FrameCountExceedsDeclaration,
                CandidateFrameFailurePosture::ProvenNoEffect,
            ));
        };
        if expected.role != frame.role()
            || expected.coordinate != frame.coordinate()
            || u64::from(expected.length) != frame.bytes().len() as u64
        {
            return Err((
                CandidateFrameContractViolation::UnexpectedFrame,
                CandidateFrameFailurePosture::ProvenNoEffect,
            ));
        }
        Ok(expected)
    }
}

#[derive(Clone, Copy)]
pub(super) struct RetainedFrameExpectation {
    declaration: CandidateFrameDeclaration,
    role: CandidateFrameRole,
    coordinate: CandidateFrameCoordinate,
    frame_bytes: u64,
    checksum: u32,
}

impl RetainedFrameExpectation {
    fn capture(declaration: CandidateFrameDeclaration, frame: &CandidateFrame) -> Self {
        Self {
            declaration,
            role: frame.role(),
            coordinate: frame.coordinate(),
            frame_bytes: frame.bytes().len() as u64,
            checksum: frame.checksum(),
        }
    }

    pub(super) const fn frame_bytes(self) -> u64 {
        self.frame_bytes
    }
}

fn verify_retained_frame(
    resident: &dyn ResidentCandidateFrame,
    expected: RetainedFrameExpectation,
) -> Result<(), CandidateFrameContractViolation> {
    if resident.bytes().len() as u64 != expected.frame_bytes
        || resident.role() != expected.role
        || resident.coordinate() != expected.coordinate
        || expected.declaration.role != expected.role
        || expected.declaration.coordinate != expected.coordinate
    {
        return Err(CandidateFrameContractViolation::RetainedFrameMismatch);
    }
    if worth_store_physical_format::durable_artifact_checksum(resident.bytes()) != expected.checksum
    {
        return Err(CandidateFrameContractViolation::RetainedFrameBytesChanged);
    }
    Ok(())
}
