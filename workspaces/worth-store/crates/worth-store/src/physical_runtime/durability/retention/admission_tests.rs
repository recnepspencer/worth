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
fn different_arenas_are_charged_separately_from_segment_generations() {
    let profile = PhysicalRetentionProfile::new(200, 4, 40, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let _segment = admission.reserve_candidate(segment(2), 60).unwrap();
    let _extent = admission
        .reserve_candidate(RecordArtifactFile::ExtentArena { arena: 1 }, 60)
        .unwrap();
    let _other = admission
        .reserve_candidate(RecordArtifactFile::ExtentArena { arena: 5 }, 40)
        .unwrap();
    assert_eq!(
        admission.lock().charged_bytes,
        160,
        "arena identities never join another artifact's charge"
    );
}

#[test]
fn physical_copy_charge_transfers_to_source_independently_of_wal_reclamation() {
    let profile = PhysicalRetentionProfile::new(32_768, 4, 4096, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let candidate = admission.reserve_retained_bytes(8192).unwrap();
    admission.reserve_retained_bytes(128).unwrap().seal();
    admission.note_sealed_publication(7, 1, 128);
    let artifact = RetiredArtifact::Extent {
        extent: 3,
        generation: 1,
        range: worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(2).unwrap(),
            4096,
            8192,
        )
        .unwrap(),
    };
    // The published root installs source retention before the live destination
    // ceases to be an excess candidate. The two owners never leave a charge gap.
    admission.retain_displaced(DisplacedArtifact {
        source_root: 4,
        artifact,
        bytes: 8192,
    });
    assert_eq!(admission.lock().charged_bytes, 16_512);
    drop(candidate);
    assert_eq!(admission.lock().charged_bytes, 8320);
    admission.release_sealed_publication(7, 1);
    assert_eq!(admission.lock().charged_bytes, 8192);
    admission.complete_displaced(artifact);
    assert_eq!(admission.lock().charged_bytes, 0);
}

#[test]
fn unpublished_copy_charge_survives_unrelated_wal_reclamation_until_owner_drop() {
    let profile = PhysicalRetentionProfile::new(16_384, 4, 4096, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let candidate = admission.reserve_retained_bytes(8192).unwrap();
    admission.reserve_retained_bytes(128).unwrap().seal();
    admission.note_sealed_publication(7, 1, 128);
    admission.release_sealed_publication(7, 1);
    assert_eq!(admission.lock().charged_bytes, 8192);
    assert!(admission.reserve_retained_bytes(4097).is_err());
    drop(candidate);
    assert_eq!(admission.lock().charged_bytes, 0);
    assert!(admission.reserve_retained_bytes(4097).is_ok());
}

#[test]
fn recovered_retirement_claim_requires_exact_garbage_and_restores_removal_permit() {
    let profile = PhysicalRetentionProfile::new(32_768, 4, 4096, 1).unwrap();
    let admission = PhysicalPublicationAdmission::new(profile);
    let displaced = DisplacedArtifact {
        source_root: 4,
        artifact: RetiredArtifact::Arena {
            arena: 2,
            generation: 4,
        },
        bytes: 8192,
    };
    assert!(admission.admit_displaced_arena(displaced));
    assert!(admission.removal_permit(displaced.artifact).is_none());
    assert!(
        !admission.claim_recovered_displaced_exact(DisplacedArtifact {
            bytes: 4096,
            ..displaced
        })
    );
    assert!(
        !admission.claim_recovered_displaced_exact(DisplacedArtifact {
            source_root: 5,
            ..displaced
        })
    );
    assert!(admission.removal_permit(displaced.artifact).is_none());
    assert!(admission.claim_recovered_displaced_exact(displaced));
    assert!(admission.removal_permit(displaced.artifact).is_some());
    admission.complete_displaced(displaced.artifact);
    assert!(!admission.claim_recovered_displaced_exact(displaced));
}

#[test]
fn reclaim_displacement_roster_is_admitted_all_or_none_before_effect() {
    let profile = PhysicalRetentionProfile::new(100, 4, 20, 1).unwrap();
    let admission = std::sync::Arc::new(PhysicalPublicationAdmission::new(profile));
    let first = admission.reserve_displaced_entries(2, 40).unwrap();
    assert_eq!(admission.lock().reserved_displaced_entries, 2);
    assert_eq!(admission.lock().reserved_displaced_bytes, 40);

    let Err(denied) = admission.reserve_displaced_entries(2, 41) else {
        panic!("a second whole roster must not enter partial capacity");
    };
    // Four total entries minus one progress entry and two reserved extents
    // leaves one, not enough for another two extents plus the active C5 head.
    assert_eq!(denied.remaining_entries, 1);
    assert_eq!(denied.remaining_bytes, 40);
    assert_eq!(admission.lock().reserved_displaced_entries, 2);
    assert_eq!(admission.lock().reserved_displaced_bytes, 40);

    drop(first);
    assert_eq!(admission.lock().reserved_displaced_entries, 0);
    assert_eq!(admission.lock().reserved_displaced_bytes, 0);
    let retry = admission.reserve_displaced_entries(2, 41).unwrap();
    drop(retry);
}
