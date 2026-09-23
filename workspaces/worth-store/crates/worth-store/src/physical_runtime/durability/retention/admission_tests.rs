use super::*;

pub(super) fn segment(generation: u64) -> RecordArtifactFile {
    RecordArtifactFile::Segment {
        segment: 1,
        generation,
    }
}

#[test]
fn growth_cannot_consume_progress_headroom() {
    let profile = PhysicalRetentionProfile::new(100, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let first = admission.reserve_candidate(segment(1), 60).unwrap();
    let Err(denied) = admission.reserve_candidate(segment(2), 1) else {
        panic!("a second generation cannot consume progress headroom");
    };
    assert_eq!(denied.remaining_bytes, 0);
    assert_eq!(denied.requested_bytes, 1);
    drop(first);
    let shared = admission.reserve_candidate(segment(7), 60).unwrap();
    let again = admission.reserve_candidate(segment(7), 60).unwrap();
    assert_eq!(admission.lock().charged_bytes, 60);
    drop(shared);
    assert_eq!(admission.lock().charged_bytes, 60);
    drop(again);
    assert_eq!(admission.lock().charged_bytes, 0);
    assert!(admission.reserve_candidate(segment(3), 61).is_err());
}

#[test]
fn equal_generations_of_different_artifacts_are_charged_separately() {
    let profile = PhysicalRetentionProfile::new(200, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let _segment = admission.reserve_candidate(segment(2), 60).unwrap();
    let _extent = admission
        .reserve_candidate(
            RecordArtifactFile::Extent {
                extent: 1,
                generation: 2,
            },
            60,
        )
        .unwrap();
    let _other = admission
        .reserve_candidate(
            RecordArtifactFile::Extent {
                extent: 5,
                generation: 2,
            },
            40,
        )
        .unwrap();
    assert_eq!(
        admission.lock().charged_bytes,
        160,
        "a shared generation number never joins another artifact's charge"
    );
}
