#![allow(dead_code)]
#[path = "../../../tests/phase_4_literal_vectors/oracle.rs"]
mod oracle;
#[path = "../../../tests/phase_4_literal_vectors/page_frames.rs"]
mod page_frames;
use super::page_frame::read_page_frame;
use crate::integrity_observation::{
    child_expectation::{ChildExpectation, ChildScope},
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
};
use worth_foundational::PhysicalArtifactFamily;

#[test]
fn independent_page_reader_accepts_each_literal_profile_and_rejects_scope_and_poison() {
    for (format, bytes) in page_frames::reader_vectors() {
        let expected = ChildExpectation {
            path: "literal.pages".into(),
            family: PhysicalArtifactFamily::PageFrame,
            generation: 11,
            format,
            offset: 0,
            length: Some(bytes.len() as u64),
            checksum: None,
            scope: ChildScope::Page {
                segment: 0x0102_0304_0506_0708,
                page: 0x1112_1314_1516_1718,
                pages: 1,
            },
        };
        assert!(read_page_frame(
            &bytes,
            &expected,
            &mut OfflineIntegrityObservationCounters::default()
        )
        .is_ok());
        let mut different = expected.clone();
        different.generation += 1;
        assert!(matches!(
            read_page_frame(
                &bytes,
                &different,
                &mut OfflineIntegrityObservationCounters::default()
            ),
            Err(OfflineIntegrityOutcome::Damaged(_))
        ));
        let mut poisoned = bytes.clone();
        let last = poisoned.len() - 1;
        poisoned[last] ^= 1;
        assert!(matches!(
            read_page_frame(
                &poisoned,
                &expected,
                &mut OfflineIntegrityObservationCounters::default()
            ),
            Err(OfflineIntegrityOutcome::Damaged(_))
        ));
    }
}
