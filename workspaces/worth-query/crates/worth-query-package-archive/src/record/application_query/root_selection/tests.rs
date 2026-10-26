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
    for tag in [0, 5, u16::MAX] {
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
