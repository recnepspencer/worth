use super::*;

pub(super) struct FrameRetentionFailure {
    cause: FrameRetentionCause,
    frame: CandidateFrame,
}

enum FrameRetentionCause {
    Contract {
        violation: CandidateFrameContractViolation,
        posture: CandidateFrameFailurePosture,
    },
    Residency {
        denial: RecordAppendDenial,
        posture: CandidateFrameFailurePosture,
    },
}

impl FrameRetentionFailure {
    fn contract(
        violation: CandidateFrameContractViolation,
        posture: CandidateFrameFailurePosture,
        frame: CandidateFrame,
    ) -> Self {
        Self {
            cause: FrameRetentionCause::Contract { violation, posture },
            frame,
        }
    }

    fn residency(
        denial: RecordAppendDenial,
        posture: CandidateFrameFailurePosture,
        frame: CandidateFrame,
    ) -> Self {
        Self {
            cause: FrameRetentionCause::Residency { denial, posture },
            frame,
        }
    }

    pub(super) fn into_write_failure<EffectFailure>(
        self,
    ) -> CandidateFrameWriteFailure<EffectFailure> {
        self.into_recoverable().into_cause()
    }

    fn into_recoverable<EffectFailure>(
        self,
    ) -> RecoverableCandidateFrameWriteFailure<EffectFailure> {
        let cause = match self.cause {
            FrameRetentionCause::Contract { violation, posture } => {
                CandidateFrameWriteFailure::Contract { violation, posture }
            }
            FrameRetentionCause::Residency { denial, posture } => {
                CandidateFrameWriteFailure::Residency { denial, posture }
            }
        };
        RecoverableCandidateFrameWriteFailure::new(cause, self.frame)
    }
}

enum SubmittedFrameWriteStop {
    Retention(FrameRetentionFailure),
    Effect(CandidateFrame),
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
        let mut effect_failure = None;
        match self.write_submitted_frame(frame, &mut |bytes| match store_write(bytes) {
            Ok(physical) => Ok(physical),
            Err(failure) => {
                let fate = failure.effect_fate();
                effect_failure = Some(failure);
                Err(fate)
            }
        }) {
            Ok(completion) => Ok(completion),
            Err(SubmittedFrameWriteStop::Retention(failure)) => Err(failure.into_recoverable()),
            Err(SubmittedFrameWriteStop::Effect(frame)) => {
                Err(RecoverableCandidateFrameWriteFailure::new(
                    CandidateFrameWriteFailure::Effect(
                        effect_failure.expect("candidate frame effect failure is stored"),
                    ),
                    frame,
                ))
            }
        }
    }

    /// One compiled candidate-frame write for every effect failure type.
    #[inline(never)]
    fn write_submitted_frame(
        &mut self,
        frame: CandidateFrame,
        store_write: &mut dyn FnMut(
            &[u8],
        ) -> Result<
            CandidateFramePhysicalWrite,
            crate::physical_runtime::PhysicalWorkEffectFate,
        >,
    ) -> Result<CandidateFrameWriteCompletion, SubmittedFrameWriteStop> {
        let (resident, expectation, frame) = self
            .retain_submitted_frame(frame)
            .map_err(SubmittedFrameWriteStop::Retention)?;
        let physical = match store_write(resident.bytes()) {
            Ok(physical) => physical,
            Err(fate) => {
                if fate == crate::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
                    && let Err(denial) = resident.discard()
                {
                    return Err(SubmittedFrameWriteStop::Retention(
                        FrameRetentionFailure::residency(
                            denial,
                            CandidateFrameFailurePosture::UnsettledBeforeEffect,
                            frame,
                        ),
                    ));
                }
                return Err(SubmittedFrameWriteStop::Effect(frame));
            }
        };
        let settlement = match physical.settle_residency(
            resident.store_identity(),
            resident.coordinate(),
            resident.bytes(),
        ) {
            Ok(settlement) => settlement,
            Err(violation) => {
                return Err(SubmittedFrameWriteStop::Retention(
                    FrameRetentionFailure::contract(
                        violation,
                        CandidateFrameFailurePosture::EffectPossible,
                        frame,
                    ),
                ))
            }
        };
        let completion = match resident.publish_clean(settlement) {
            Ok(completion) => completion,
            Err(denial) => {
                return Err(SubmittedFrameWriteStop::Retention(
                    FrameRetentionFailure::residency(
                        denial,
                        CandidateFrameFailurePosture::EffectPossible,
                        frame,
                    ),
                ))
            }
        };
        self.complete_frame(expectation, &completion, frame)
            .map_err(SubmittedFrameWriteStop::Retention)?;
        Ok(completion)
    }

    pub(super) fn retain_submitted_frame(
        &mut self,
        frame: CandidateFrame,
    ) -> Result<
        (
            Box<dyn ResidentCandidateFrame>,
            RetainedFrameExpectation,
            CandidateFrame,
        ),
        FrameRetentionFailure,
    > {
        let expected = match self.validate_submitted_frame(&frame) {
            Ok(expected) => expected,
            Err((violation, posture)) => {
                return Err(FrameRetentionFailure::contract(violation, posture, frame))
            }
        };
        let expectation = RetainedFrameExpectation::capture(expected, &frame);
        let next_bytes = self.resident_bytes.saturating_add(expectation.frame_bytes);
        if next_bytes > self.declaration.total_frame_bytes() {
            return Err(FrameRetentionFailure::contract(
                CandidateFrameContractViolation::FrameBytesExceedDeclaration,
                CandidateFrameFailurePosture::ProvenNoEffect,
                frame,
            ));
        }
        let resident = match self.residency.retain(&frame) {
            Ok(resident) => resident,
            Err(denial) => {
                return Err(FrameRetentionFailure::residency(
                    denial,
                    CandidateFrameFailurePosture::ProvenNoEffect,
                    frame,
                ))
            }
        };
        if let Err(violation) = verify_retained_frame(resident.as_ref(), expectation) {
            if let Err(denial) = resident.discard() {
                return Err(FrameRetentionFailure::residency(
                    denial,
                    CandidateFrameFailurePosture::UnsettledBeforeEffect,
                    frame,
                ));
            }
            return Err(FrameRetentionFailure::contract(
                violation,
                CandidateFrameFailurePosture::ProvenNoEffect,
                frame,
            ));
        }
        Ok((resident, expectation, frame))
    }

    pub(super) fn complete_frame(
        &mut self,
        expectation: RetainedFrameExpectation,
        completion: &CandidateFrameWriteCompletion,
        frame: CandidateFrame,
    ) -> Result<(), FrameRetentionFailure> {
        if completion.frame_bytes() != expectation.frame_bytes {
            return Err(FrameRetentionFailure::contract(
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
