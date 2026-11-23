//! Round-trip and malformed-byte refusal proofs for `ApplicationNameScheme` (tag 57)
//! and `ApplicationGroupNameScheme` (tag 58).
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationNameRoleDependency, ApplicationNameSubjectKind,
    ErasedApplicationGroupNameScheme, ErasedApplicationNameScheme,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaMember as Member, WorthQueryPortableApplicationSchemaParts,
    WorthQueryPortableApplicationSchemaRecord,
};
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;
use crate::record::decode_budget::RecordDecodeAttempt;

use super::{decode_member, encode_member, encode_payload, try_decode_member};

fn populated_scheme(kind: ApplicationNameSubjectKind, subject: &str) -> Member {
    Member::ApplicationNameScheme {
        definition: ErasedApplicationNameScheme::from_untrusted_parts(
            format!("worth.cad.{}.v1", subject.to_lowercase()),
            kind,
            subject.to_owned(),
            vec![
                "authored".to_owned(),
                "cut_piece".to_owned(),
                "seam_patch".to_owned(),
            ],
            vec![
                ApplicationNameRoleDependency::new("cut_piece", "authored"),
                ApplicationNameRoleDependency::new("seam_patch", "cut_piece"),
            ],
            true,
            Some("worth.cad.face.split.v1".to_owned()),
            vec!["authored".to_owned(), "cut_piece".to_owned()],
            true,
        ),
    }
}

#[test]
fn application_name_scheme_round_trips_every_field_and_subject_kind() {
    for (kind, subject) in [
        (ApplicationNameSubjectKind::Entity, "Face"),
        (ApplicationNameSubjectKind::Relation, "Seam"),
        (ApplicationNameSubjectKind::Group, "FaceLoop"),
    ] {
        let member = populated_scheme(kind, subject);
        let bytes = encode_member(&member);
        assert_eq!(&bytes[..4], &[0, 57, 0, 1]);
        assert_eq!(decode_member(&bytes), member);
    }

    let minimal = Member::ApplicationNameScheme {
        definition: ErasedApplicationNameScheme::from_untrusted_parts(
            "worth.cad.edge.v1".to_owned(),
            ApplicationNameSubjectKind::Entity,
            "Edge".to_owned(),
            vec!["authored".to_owned()],
            Vec::new(),
            false,
            None,
            Vec::new(),
            false,
        ),
    };
    assert_eq!(decode_member(&encode_member(&minimal)), minimal);

    // Full portable schema payload round-trip containing Entity + ApplicationNameScheme.
    let mut members = vec![
        Member::Entity {
            entity: "Face".to_owned(),
        },
        populated_scheme(ApplicationNameSubjectKind::Entity, "Face"),
    ];
    members.sort();
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "worth.tests.archive".to_owned(),
            name: "NamingArchive".to_owned(),
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
        },
    );
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let payload = encode_payload(&record, limits);
    let mut input = BinaryInput::new(&payload);
    let mut attempt =
        RecordDecodeAttempt::begin(Default::default(), payload.len() as u64, limits).unwrap();
    let WorthQueryPortablePackageRecord::ApplicationSchema(decoded) =
        super::super::decode_payload(&mut input, &mut attempt).unwrap()
    else {
        panic!("expected application schema payload");
    };
    assert!(input.is_finished());
    assert_eq!(decoded, record);
}

#[test]
fn application_name_scheme_refuses_malformed_version_subject_kind_bool_and_truncation() {
    // 1. Unknown local record version != 1.
    assert_eq!(
        try_decode_member(&[0, 57, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVersion
    );

    // 2. Invalid subject_kind byte outside {1, 2, 3}.
    for bad_kind in [0_u8, 4_u8] {
        let mut out = BinaryOutput::with_capacity(64);
        out.u16(57);
        out.u16(1);
        out.text("worth.cad.bad.v1");
        out.raw_bytes(&[bad_kind]);
        assert_eq!(
            try_decode_member(&out.into_bytes()).unwrap_err().kind(),
            Kind::InvalidRecordShape
        );
    }

    // 3. Invalid boolean bytes outside {0, 1} on split_capable, has_disc, require_authored.
    let valid = encode_member(&populated_scheme(
        ApplicationNameSubjectKind::Entity,
        "Face",
    ));
    // Locate the trailing `require_authored_occurrence` byte (last byte == 1) and corrupt it to 2.
    let mut bad_authored = valid.clone();
    *bad_authored.last_mut().unwrap() = 2;
    assert_eq!(
        try_decode_member(&bad_authored).unwrap_err().kind(),
        Kind::InvalidBooleanEncoding
    );

    // Construct exact wire with bad split_capable = 2 and bad has_discriminator = 2.
    for (split_byte, has_disc_byte) in [(2_u8, 0_u8), (1_u8, 2_u8)] {
        let mut out = BinaryOutput::with_capacity(128);
        out.u16(57);
        out.u16(1);
        out.text("worth.cad.badbool.v1");
        out.raw_bytes(&[1]);
        out.text("Face");
        out.u32(1);
        out.text("authored");
        out.u32(0);
        out.raw_bytes(&[split_byte, has_disc_byte]);
        out.u32(0);
        out.raw_bytes(&[0]);
        assert_eq!(
            try_decode_member(&out.into_bytes()).unwrap_err().kind(),
            Kind::InvalidBooleanEncoding
        );
    }

    // 4. Truncated bytes refuse cleanly.
    for cut in [4, 10, valid.len() - 1] {
        assert_eq!(
            try_decode_member(&valid[..cut]).unwrap_err().kind(),
            Kind::Truncated
        );
    }

    // 5. Nesting depth 1 is refused by tag 57.
    let mut input = BinaryInput::new(&valid);
    let narrow = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1);
    let mut attempt =
        RecordDecodeAttempt::begin(Default::default(), valid.len() as u64, narrow).unwrap();
    assert_eq!(
        super::super::member::decode(&mut input, &mut attempt)
            .unwrap_err()
            .kind(),
        Kind::NestingDepthBudgetExceeded
    );
}

fn populated_group_scheme(
    rule: ApplicationGroupContinuityRule,
    subject: &str,
    anchor_role: Option<&str>,
    boundary_role: Option<&str>,
) -> Member {
    Member::ApplicationGroupNameScheme {
        definition: ErasedApplicationGroupNameScheme::from_untrusted_parts(
            ErasedApplicationNameScheme::from_untrusted_parts(
                format!("worth.cad.group.{}.v1", subject.to_lowercase()),
                ApplicationNameSubjectKind::Group,
                subject.to_owned(),
                vec!["host".to_owned(), "wall".to_owned(), "member".to_owned()],
                Vec::new(),
                false,
                None,
                Vec::new(),
                true,
            ),
            rule,
            anchor_role.map(str::to_owned),
            boundary_role.map(str::to_owned),
        ),
    }
}

#[test]
fn application_group_name_scheme_round_trips_all_continuity_rules_and_payload() {
    for (rule, subject, anchor, boundary) in [
        (
            ApplicationGroupContinuityRule::Anchor,
            "FaceLoop",
            Some("host"),
            None,
        ),
        (
            ApplicationGroupContinuityRule::Boundary,
            "RoomSpace",
            None,
            Some("wall"),
        ),
        (
            ApplicationGroupContinuityRule::ProducerRole,
            "BodyShell",
            Some("host"),
            Some("wall"),
        ),
    ] {
        let member = populated_group_scheme(rule, subject, anchor, boundary);
        let bytes = encode_member(&member);
        assert_eq!(&bytes[..4], &[0, 58, 0, 1]);
        assert_eq!(decode_member(&bytes), member);
    }

    let mut members = vec![
        Member::Entity {
            entity: "Face".to_owned(),
        },
        populated_group_scheme(
            ApplicationGroupContinuityRule::Boundary,
            "RoomSpace",
            None,
            Some("wall"),
        ),
    ];
    members.sort();
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "worth.tests.archive".to_owned(),
            name: "GroupNamingArchive".to_owned(),
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
        },
    );
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let payload = encode_payload(&record, limits);
    let mut input = BinaryInput::new(&payload);
    let mut attempt =
        RecordDecodeAttempt::begin(Default::default(), payload.len() as u64, limits).unwrap();
    let WorthQueryPortablePackageRecord::ApplicationSchema(decoded) =
        super::super::decode_payload(&mut input, &mut attempt).unwrap()
    else {
        panic!("expected application schema payload");
    };
    assert!(input.is_finished());
    assert_eq!(decoded, record);
}

#[test]
fn application_group_name_scheme_refuses_tampered_version_rule_bool_truncation_and_depth() {
    assert_eq!(
        try_decode_member(&[0, 58, 0, 2]).unwrap_err().kind(),
        Kind::UnsupportedRecordVersion
    );

    let valid = encode_member(&populated_group_scheme(
        ApplicationGroupContinuityRule::ProducerRole,
        "BodyShell",
        Some("host"),
        Some("wall"),
    ));
    // Corrupt continuity-rule byte (immediately before the optional anchor_role flag) to 0 and 4.
    let prefix_len = encode_member(&populated_group_scheme(
        ApplicationGroupContinuityRule::Anchor,
        "BodyShell",
        None,
        None,
    ))
    .len()
        - 3; // trailing 3 bytes are: continuity_rule(1), anchor_flag(0), boundary_flag(0)
    for bad_rule in [0_u8, 4_u8] {
        let mut tampered = valid.clone();
        tampered[prefix_len] = bad_rule;
        assert_eq!(
            try_decode_member(&tampered).unwrap_err().kind(),
            Kind::InvalidRecordShape
        );
    }

    // Corrupt optional anchor_role presence boolean to 2.
    let mut bad_opt = valid.clone();
    bad_opt[prefix_len + 1] = 2;
    assert_eq!(
        try_decode_member(&bad_opt).unwrap_err().kind(),
        Kind::InvalidBooleanEncoding
    );

    for cut in [4, prefix_len + 1, valid.len() - 1] {
        assert_eq!(
            try_decode_member(&valid[..cut]).unwrap_err().kind(),
            Kind::Truncated
        );
    }

    let mut input = BinaryInput::new(&valid);
    let narrow = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1);
    let mut attempt =
        RecordDecodeAttempt::begin(Default::default(), valid.len() as u64, narrow).unwrap();
    assert_eq!(
        super::super::member::decode(&mut input, &mut attempt)
            .unwrap_err()
            .kind(),
        Kind::NestingDepthBudgetExceeded
    );
}
