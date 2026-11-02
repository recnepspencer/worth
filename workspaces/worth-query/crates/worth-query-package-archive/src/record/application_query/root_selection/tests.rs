//! The combined discriminator is closed and preserves old one-hop byte width.
use super::*;
use crate::binary_output::BinaryOutput;
#[test]
fn exact_root_path_tags_round_trip_and_unknown_modes_refuse() {
    for (tag, traversal, direction) in [
        (1, Traversal::OneHop, RelationDirection::Forward),
        (2, Traversal::OneHop, RelationDirection::Reverse),
        (3, Traversal::Closure, RelationDirection::Forward),
        (4, Traversal::Closure, RelationDirection::Reverse),
    ] {
        let step = ApplicationQueryRootPathStep::from_untrusted_fields(
            "Next".into(),
            "Node".into(),
            "Node".into(),
            direction,
            traversal,
        );
        let mut output = BinaryOutput::with_capacity(26);
        write_step(&mut output, &step).unwrap();
        let bytes = output.into_bytes();
        assert_eq!(bytes.len(), 26);
        assert_eq!(&bytes[24..], &u16::to_be_bytes(tag));
        let mut input = BinaryInput::new(&bytes);
        assert_eq!(decode_step(&mut input).unwrap(), step);
        assert!(input.is_finished());
    }
    for tag in [0, 7, u16::MAX] {
        let mut output = BinaryOutput::with_capacity(26);
        output.text("Next");
        output.text("Node");
        output.text("Node");
        output.u16(tag);
        let bytes = output.into_bytes();
        assert_eq!(
            decode_step(&mut BinaryInput::new(&bytes))
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}

#[test]
fn derived_step_tags_are_distinct_and_old_wire_is_literal() {
    for tag in 1u16..=6 {
        let kind = if tag <= 4 {
            SourceKind::Stored
        } else {
            SourceKind::Derived
        };
        let traversal = if tag == 3 || tag == 4 {
            Traversal::Closure
        } else {
            Traversal::OneHop
        };
        let direction = if tag % 2 == 1 {
            RelationDirection::Forward
        } else {
            RelationDirection::Reverse
        };
        let step = ApplicationQueryRootPathStep::from_untrusted_source_fields(
            "Next".into(),
            "Node".into(),
            "Node".into(),
            kind,
            direction,
            traversal,
        );
        let mut bytes = vec![
            0, 0, 0, 4, b'N', b'e', b'x', b't', 0, 0, 0, 4, b'N', b'o', b'd', b'e', 0, 0, 0, 4,
            b'N', b'o', b'd', b'e', 0, 0,
        ];
        bytes[25] = tag as u8;
        let mut output = BinaryOutput::with_capacity(26);
        write_step(&mut output, &step).unwrap();
        assert_eq!(output.into_bytes(), bytes);
        assert_eq!(decode_step(&mut BinaryInput::new(&bytes)).unwrap(), step);
    }
}
#[test]
fn unsupported_derived_closure_and_projected_namespaces_refuse_before_step_bytes() {
    use worth_query_declaration::facade::application_schema::ApplicationGraphMemberMeaning as Member;
    for (from, to, mode) in [
        (
            Member::Entity("Node".into()),
            Member::Entity("Node".into()),
            Traversal::Closure,
        ),
        (
            Member::Projected("Node".into()),
            Member::Entity("Node".into()),
            Traversal::OneHop,
        ),
        (
            Member::Entity("Node".into()),
            Member::Projected("Node".into()),
            Traversal::OneHop,
        ),
    ] {
        let step = ApplicationQueryRootPathStep::from_untrusted_source_fields(
            "Live".into(),
            from,
            to,
            SourceKind::Derived,
            RelationDirection::Forward,
            mode,
        );
        let mut output = BinaryOutput::with_capacity(0);
        assert_eq!(
            write_step(&mut output, &step).unwrap_err().kind(),
            Kind::InvalidRecordShape
        );
        assert!(output.into_bytes().is_empty());
    }
}
