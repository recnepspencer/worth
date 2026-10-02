use super::*;

#[path = "tests/source.rs"]
mod source;
use source::{admitted_source, admitted_source_with_cutoff, record_format};

#[test]
fn v2_roster_and_rooted_walk_share_one_admission_budget_before_head_read() {
    let source = admitted_source();
    let format = record_format();
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(8, 8, 8 * 16_384, 256 * 1024, 4).unwrap();
    let generous = 256 * 1024;
    let mut positive_reads = 0;
    let admitted = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        limits,
        8,
        generous,
        |reference, _| {
            positive_reads += 1;
            assert_eq!(reference, source.head.reference(format));
            Ok::<_, ()>(source.head.encode(format))
        },
    )
    .expect("the exact V2 checkpoint and rooted source head admit");
    assert_eq!(positive_reads, 1);
    assert_eq!(admitted.selected_heads(), &[source.entry]);
    assert_eq!(
        admitted.accumulator_v2().head_roster_digest(),
        source.digest
    );
    assert_eq!(admitted.accumulator_v2().head_count(), 1);
    let owned = admitted.owned_heap_bytes().unwrap();
    let walker = admitted.head_walk_peak_resident_bytes();
    assert!(owned > 0 && walker > 0);
    let insufficient = owned + walker - 1;
    assert!(insufficient > owned && insufficient > walker);
    let mut denied_reads = 0;
    let denied = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &source.checkpoint,
        &source.root,
        format,
        limits,
        8,
        insufficient,
        |_, _| {
            denied_reads += 1;
            Ok::<_, ()>(source.head.encode(format))
        },
    );
    assert!(matches!(
        denied,
        Err(SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::ResidentBoundExceeded { required, admitted }))
            if required > admitted && admitted == insufficient
    ));
    assert_eq!(denied_reads, 0);
    assert!(admitted.admission_peak_resident_bytes() <= generous);
}

#[test]
fn different_valid_checkpoint_stream_denies_before_rooted_head_read() {
    let source = admitted_source();
    let other = admitted_source_with_cutoff(19);
    assert_eq!(source.checkpoint.source(), other.checkpoint.source());
    assert_ne!(source.checkpoint.facts(), other.checkpoint.facts());
    let format = record_format();
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(8, 8, 8 * 16_384, 256 * 1024, 4).unwrap();
    let mut reads = 0;
    let denied = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        &source.selected,
        &other.checkpoint,
        &source.root,
        format,
        limits,
        8,
        256 * 1024,
        |_, _| {
            reads += 1;
            Ok::<_, ()>(source.head.encode(format))
        },
    );
    assert!(matches!(
        denied,
        Err(SelectedHeadRosterAdmissionDenial::Custody(
            SelectedCustodyDenial::CertificateRoster
        ))
    ));
    assert_eq!(reads, 0);
}
