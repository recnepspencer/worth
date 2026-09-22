use super::*;
use crate::compatibility::WorthQueryPackageArchiveCompatibilityPosture;

#[test]
fn duplicate_semantic_facts_round_trip_as_a_canonical_multiset() {
    let mut output = BinaryOutput::with_capacity(128);
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION);
    output.text("program.v1");
    output.text("revision");
    output.u32(2);
    for _ in 0..2 {
        output.raw_bytes(&[encode_family(ApplicationSemanticFamily::Rules)]);
        output.text("same-subject");
        output.text("same-meaning");
    }

    let decoded = decode_application_program_description(
        &output.into_bytes(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .expect("canonical duplicate facts retain multiset meaning");
    assert_eq!(decoded.facts().len(), 2);
    assert_eq!(decoded.facts()[0], decoded.facts()[1]);
}

#[test]
fn descending_semantic_facts_are_not_normalized() {
    let mut output = BinaryOutput::with_capacity(128);
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION);
    output.text("program.v1");
    output.text("revision");
    output.u32(2);
    for subject in ["z", "a"] {
        output.raw_bytes(&[encode_family(ApplicationSemanticFamily::Rules)]);
        output.text(subject);
        output.text("meaning");
    }

    let denial = decode_application_program_description(
        &output.into_bytes(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .expect_err("descending facts cannot be normalized by the decoder");
    assert_eq!(denial.kind(), Kind::NonCanonicalRecordSequence);
}

#[test]
fn unsupported_program_description_version_reports_its_exact_layer() {
    for (version, posture) in [
        (0, WorthQueryPackageArchiveCompatibilityPosture::InvalidZero),
        (
            WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION + 1,
            WorthQueryPackageArchiveCompatibilityPosture::ExceedsWindow,
        ),
    ] {
        let mut output = BinaryOutput::with_capacity(6);
        output.raw_bytes(MAGIC);
        output.u16(version);
        let denial = decode_application_program_description(
            &output.into_bytes(),
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .expect_err("unsupported version denies before interpreting a body");
        assert_eq!(
            denial.kind(),
            Kind::UnsupportedApplicationProgramDescriptionVersion
        );
        let compatibility = denial.compatibility().expect("typed compatibility posture");
        assert_eq!(
            compatibility.layer(),
            WorthQueryPackageArchiveProtocolLayer::ApplicationProgramDescription
        );
        assert_eq!(compatibility.observed_version(), version);
        assert_eq!(compatibility.posture(), posture);
    }
}

#[test]
fn hostile_fact_count_is_rejected_before_fact_allocation() {
    let mut output = BinaryOutput::with_capacity(32);
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION);
    output.text("program.v1");
    output.text("revision");
    output.u32(
        u32::try_from(WorthQueryPackageArchiveLimits::DEFAULT.maximum_nested_entries())
            .expect("the default fact ceiling fits the wire width"),
    );

    let denial = decode_application_program_description(
        &output.into_bytes(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .expect_err("a count without its minimum fact framing is truncated");
    assert_eq!(denial.kind(), Kind::Truncated);
}

#[test]
fn unknown_semantic_family_cannot_enter_a_description() {
    let mut output = BinaryOutput::with_capacity(64);
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION);
    output.text("program.v1");
    output.text("revision");
    output.u32(1);
    output.raw_bytes(&[0]);
    output.text("");
    output.text("");

    let denial = decode_application_program_description(
        &output.into_bytes(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .expect_err("an unknown semantic family remains unsupported");
    assert_eq!(denial.kind(), Kind::UnsupportedRecordVariant);
}
