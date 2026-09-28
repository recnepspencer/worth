use super::*;
use crate::physical_runtime::record_serving::residency::candidate_frame_publishers::{
    BoundedCandidateFramePublisher, CandidateFrameCounterCells,
};
use worth_store_buffer_pool::ForegroundWriteAllocationGrant;

#[path = "tests/allocation_authority.rs"]
mod allocation_authority;
#[path = "tests/effect_fate_cleanup.rs"]
mod effect_fate_cleanup;
#[path = "tests/exact_receipt.rs"]
mod exact_receipt;

use super::write_progression::CandidateFrameEffectFailure;
use allocation_authority::{publication_allocation, test_pool};

struct PreEffectFailure;

#[test]
fn arena_manifest_is_a_range_candidate_not_a_complete_artifact() {
    let coordinate =
        CandidateFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 7 }, 8192);
    assert!(!CandidateFrameRole::ExtentManifest.is_complete_artifact());
    let declaration =
        CandidateFrameDeclaration::new(CandidateFrameRole::ExtentManifest, coordinate, 104)
            .unwrap();
    let candidate = CandidateFrameSet::new(1, vec![declaration]).unwrap();
    assert_eq!(candidate.declarations()[0].coordinate(), coordinate);
    assert!(CandidateFrameSet::new(
        1,
        vec![
            CandidateFrameDeclaration::new(CandidateFrameRole::ManifestBlock, coordinate, 104)
                .unwrap()
        ]
    )
    .is_none());
}

impl CandidateFrameEffectFailure for PreEffectFailure {
    fn effect_fate(&self) -> crate::physical_runtime::PhysicalWorkEffectFate {
        unreachable!("pre-effect test closures never produce an effect failure")
    }
}

fn segment_coordinate(offset: u64) -> CandidateFrameCoordinate {
    CandidateFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 1,
        },
        offset,
    )
}

fn session<'allocation>(
    allocation: &'allocation ForegroundWriteAllocationGrant,
    declaration: CandidateFrameSet,
) -> StoreCandidateFramePublicationSession<'allocation> {
    let port = PreEffectPublisher;
    StoreCandidateFramePublicationSession::begin(&port, allocation, declaration).unwrap()
}

fn declared_inline_frames(frames: &[(u64, u32)]) -> CandidateFrameSet {
    CandidateFrameSet::new(
        1,
        frames
            .iter()
            .map(|(offset, length)| {
                CandidateFrameDeclaration::new(
                    CandidateFrameRole::InlinePage,
                    segment_coordinate(*offset),
                    *length,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn declaration_and_coordinate_violations_precede_store_writes() {
    let pool = test_pool(102);
    let allocation = publication_allocation(&pool);
    let mut oversized = session(&allocation, declared_inline_frames(&[(0, 1)]));
    let mut writes = 0;
    let failure = oversized
        .write_frame(
            CandidateFrame::new(
                CandidateFrameRole::InlinePage,
                segment_coordinate(0),
                vec![1, 2],
            ),
            &mut |_| -> Result<CandidateFramePhysicalWrite, PreEffectFailure> {
                writes += 1;
                panic!("declaration rejection must precede the physical effect")
            },
        )
        .unwrap_err();
    assert!(matches!(
        failure,
        CandidateFrameWriteFailure::Contract {
            violation: CandidateFrameContractViolation::UnexpectedFrame,
            posture: CandidateFrameFailurePosture::ProvenNoEffect,
        }
    ));
    let mut mismatched = session(&allocation, declared_inline_frames(&[(0, 1)]));
    let failure = mismatched
        .write_frame(
            CandidateFrame::new(
                CandidateFrameRole::CatalogCandidate,
                segment_coordinate(0),
                vec![1],
            ),
            &mut |_| -> Result<CandidateFramePhysicalWrite, PreEffectFailure> {
                writes += 1;
                panic!("coordinate rejection must precede the physical effect")
            },
        )
        .unwrap_err();
    assert!(matches!(
        failure,
        CandidateFrameWriteFailure::Contract {
            violation: CandidateFrameContractViolation::CoordinateRoleMismatch,
            posture: CandidateFrameFailurePosture::ProvenNoEffect,
        }
    ));
    assert_eq!(writes, 0);
}

#[test]
fn retained_bytes_must_still_be_the_declared_candidate_before_any_store_effect() {
    let pool = test_pool(104);
    let allocation = publication_allocation(&pool);
    let port = MutatingPublisher;
    let mut session = StoreCandidateFramePublicationSession::begin(
        &port,
        &allocation,
        declared_inline_frames(&[(0, 3)]),
    )
    .unwrap();
    let mut writes = 0;
    let failure = session
        .write_frame(
            CandidateFrame::new(
                CandidateFrameRole::InlinePage,
                segment_coordinate(0),
                vec![1, 2, 3],
            ),
            &mut |_| -> Result<CandidateFramePhysicalWrite, PreEffectFailure> {
                writes += 1;
                panic!("retained-byte validation must precede the physical effect")
            },
        )
        .unwrap_err();
    assert!(matches!(
        failure,
        CandidateFrameWriteFailure::Contract {
            violation: CandidateFrameContractViolation::RetainedFrameBytesChanged,
            posture: CandidateFrameFailurePosture::ProvenNoEffect,
        }
    ));
    assert_eq!(writes, 0);
}

struct MutatingPublisher;

impl CandidateFramePublicationPort for MutatingPublisher {
    fn begin<'allocation>(
        &self,
        _: &'allocation ForegroundWriteAllocationGrant,
        _: &CandidateFrameSet,
    ) -> Result<Box<dyn CandidateFrameResidencySession + 'allocation>, RecordAppendDenial> {
        Ok(Box::new(MutatingSession))
    }
}

struct MutatingSession;

impl CandidateFrameResidencySession for MutatingSession {
    fn retain(
        &mut self,
        frame: &CandidateFrame,
    ) -> Result<Box<dyn ResidentCandidateFrame>, RecordAppendDenial> {
        let mut retained =
            CandidateFrame::new(frame.role(), frame.coordinate(), frame.bytes().to_vec());
        retained.bytes[0] ^= 0xff;
        Ok(Box::new(MutatingResident(retained)))
    }
}

struct MutatingResident(CandidateFrame);

impl ResidentCandidateFrame for MutatingResident {
    fn store_identity(&self) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        unreachable!("retained-byte validation precedes Store settlement")
    }

    fn role(&self) -> CandidateFrameRole {
        self.0.role()
    }
    fn coordinate(&self) -> CandidateFrameCoordinate {
        self.0.coordinate()
    }
    fn bytes(&self) -> &[u8] {
        self.0.bytes()
    }
    fn discard(self: Box<Self>) -> Result<(), RecordAppendDenial> {
        Ok(())
    }
    fn into_dirty(
        self: Box<Self>,
    ) -> Result<worth_store_buffer_pool::DirtyPhysicalFrame, RecordAppendDenial> {
        Err(RecordAppendDenial::from_residency(
            worth_store_buffer_pool::PhysicalResidencyDenial::FrameNotResident,
        ))
    }
    fn publish_clean(
        self: Box<Self>,
        _settlement: CandidateFrameResidencySettlement,
    ) -> Result<CandidateFrameWriteCompletion, RecordAppendDenial> {
        Ok(CandidateFrameWriteCompletion::retained(
            self.0.bytes().len() as u64,
        ))
    }
}

struct PreEffectPublisher;

impl CandidateFramePublicationPort for PreEffectPublisher {
    fn begin<'allocation>(
        &self,
        _: &'allocation ForegroundWriteAllocationGrant,
        _: &CandidateFrameSet,
    ) -> Result<Box<dyn CandidateFrameResidencySession + 'allocation>, RecordAppendDenial> {
        Ok(Box::new(PreEffectSession))
    }
}

struct PreEffectSession;

impl CandidateFrameResidencySession for PreEffectSession {
    fn retain(
        &mut self,
        _: &CandidateFrame,
    ) -> Result<Box<dyn ResidentCandidateFrame>, RecordAppendDenial> {
        panic!("pre-effect declaration tests must not retain a candidate")
    }
}
