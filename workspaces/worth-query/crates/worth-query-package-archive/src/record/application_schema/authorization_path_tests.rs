//! Legacy authorization paths keep their bytes; extended paths round-trip
//! under tags 3/4, which a reader of the legacy layout refuses.
use worth_foundational::facade::AspectValue;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationComparison as Comparison, ApplicationAuthorizationFieldConstraint,
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect as Effect,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
    ApplicationAuthorizationTraversalDirection as Direction,
    WorthQueryPortableApplicationAuthorizationFieldConstraintParts as ConstraintParts,
    WorthQueryPortableApplicationAuthorizationPathParts as PathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts as PredicateParts,
    WorthQueryPortableApplicationAuthorizationTraversalParts as TraversalParts,
};

use super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::foundational_value;
use super::super::super::sequence::write_sequence;
use super::{decode, write, write_traversal};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::facade::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;

fn predicate(comparison: Comparison) -> ApplicationAuthorizationPredicate {
    ApplicationAuthorizationPredicate::from_untrusted_parts(
        ARCHIVE_DECODE,
        PredicateParts {
            traversal_ordinal: 1,
            entity: "Assignment".into(),
            aspect: "AssignmentFacts".into(),
            field: "role".into(),
            value: AspectValue::UInt64(2),
            comparison,
        },
    )
}

fn path(
    effect: Effect,
    predicates: Vec<ApplicationAuthorizationPredicate>,
    field_constraints: Vec<ApplicationAuthorizationFieldConstraint>,
) -> ApplicationAuthorizationPath {
    let traversal = ApplicationAuthorizationTraversal::from_untrusted_parts(
        ARCHIVE_DECODE,
        TraversalParts {
            relation: "UserAssignments".into(),
            from: "User".into(),
            to: "Assignment".into(),
            direction: Direction::Forward,
        },
    );
    ApplicationAuthorizationPath::from_untrusted_parts(
        ARCHIVE_DECODE,
        PathParts {
            effect,
            principal_entity: "User".into(),
            scope_entity: "Assignment".into(),
            traversals: vec![traversal],
            predicates,
            field_constraints,
        },
    )
}

fn constraint() -> ApplicationAuthorizationFieldConstraint {
    ApplicationAuthorizationFieldConstraint::from_untrusted_parts(
        ARCHIVE_DECODE,
        ConstraintParts {
            traversal_ordinal: 1,
            entity: "Assignment".into(),
            aspect: "AssignmentFacts".into(),
            field: "granted_by".into(),
            comparison: Comparison::StrictlyLess,
            principal_aspect: "UserFacts".into(),
            principal_field: "user_key".into(),
        },
    )
}

fn encoded(path: &ApplicationAuthorizationPath) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(256);
    assert!(write(&mut output, path).is_ok());
    output.into_bytes()
}

fn decoded(bytes: &[u8]) -> Result<ApplicationAuthorizationPath, Kind> {
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let Ok(mut budget) = RecordDecodeAttempt::begin(Default::default(), bytes.len() as u64, limits)
    else {
        return Err(Kind::Truncated);
    };
    let mut input = BinaryInput::new(bytes);
    let path = decode(&mut input, &mut budget).map_err(|denial| denial.kind())?;
    assert!(input.is_finished(), "the whole path encoding is consumed");
    Ok(path)
}

/// The exact layout written before comparisons existed.
fn legacy_layout(path: &ApplicationAuthorizationPath, tag: u16) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(256);
    let parts = path.parts();
    let write_legacy = |output: &mut dyn BinaryEncodingSink| -> Result<(), crate::denial::WorthQueryPackageArchiveDenial> {
        output.u16(tag)?;
        output.text(&parts.principal_entity)?;
        output.text(&parts.scope_entity)?;
        write_sequence(output, &parts.traversals, |output, traversal| {
            write_traversal(output, traversal)
        })?;
        write_sequence(output, &parts.predicates, |output, predicate| {
            let parts = predicate.parts();
            super::super::wire_vocabulary::write_usize(output, parts.traversal_ordinal)?;
            output.text(&parts.entity)?;
            output.text(&parts.aspect)?;
            output.text(&parts.field)?;
            foundational_value::write_aspect_value(output, &parts.value)
        })
    };
    assert!(write_legacy(&mut output).is_ok());
    output.into_bytes()
}

#[test]
fn equality_paths_keep_their_legacy_bytes_and_tags() {
    for (effect, tag) in [(Effect::Allow, 1), (Effect::Deny, 2)] {
        let legacy = path(effect, vec![predicate(Comparison::Equal)], Vec::new());
        assert!(!legacy.uses_extended_conditions());
        let bytes = encoded(&legacy);
        assert_eq!(bytes, legacy_layout(&legacy, tag));
        assert_eq!(decoded(&bytes), Ok(legacy));
    }
}

#[test]
fn comparisons_and_principal_constraints_round_trip_under_tags_three_and_four() {
    for (effect, tag) in [(Effect::Allow, 3_u16), (Effect::Deny, 4)] {
        let extended = path(
            effect,
            vec![predicate(Comparison::Equal), predicate(Comparison::AtLeast)],
            vec![constraint()],
        );
        let bytes = encoded(&extended);
        assert_eq!(
            bytes[..2],
            tag.to_be_bytes(),
            "extended paths use the new tags"
        );
        assert_eq!(decoded(&bytes), Ok(extended));
    }
    let constraint_only = path(Effect::Allow, Vec::new(), vec![constraint()]);
    assert_eq!(decoded(&encoded(&constraint_only)), Ok(constraint_only));
}

#[test]
fn a_legacy_reader_refuses_the_extended_tags() {
    // The tag table of the reader released before tags 3 and 4.
    let legacy_reader = |tag: u16| matches!(tag, 1 | 2);
    let extended = path(
        Effect::Allow,
        vec![predicate(Comparison::AtMost)],
        Vec::new(),
    );
    let bytes = encoded(&extended);
    assert!(!legacy_reader(u16::from_be_bytes([bytes[0], bytes[1]])));
}

#[test]
fn non_canonical_and_mismatched_extended_encodings_refuse() {
    // Tag 3 over an equality-only path: one meaning must have one encoding.
    let legacy = path(
        Effect::Allow,
        vec![predicate(Comparison::Equal)],
        Vec::new(),
    );
    let mut bytes = legacy_layout(&legacy, 3);
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(decoded(&bytes), Err(Kind::UnsupportedRecordVariant));

    // One comparison per predicate, no more and no fewer.
    let mut short = legacy_layout(&legacy, 3);
    short.extend_from_slice(&0_u32.to_be_bytes());
    short.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(decoded(&short), Err(Kind::UnsupportedRecordVariant));

    // An unknown comparison tag is refused.
    let mut unknown = legacy_layout(&legacy, 3);
    unknown.extend_from_slice(&1_u32.to_be_bytes());
    unknown.extend_from_slice(&9_u16.to_be_bytes());
    unknown.extend_from_slice(&0_u32.to_be_bytes());
    assert_eq!(decoded(&unknown), Err(Kind::UnsupportedRecordVariant));
}
