//! Old field target bytes remain exact; parent removal owns an appended tag.
use super::*;
#[test]
fn aspect_removal_target_does_not_reinterpret_existing_field_write_tag() {
    let field = ApplicationOperationProgramTarget::Write {
        entity: "E".into(),
        aspect: "A".into(),
        field: "F".into(),
    };
    let removal = ApplicationOperationProgramTarget::RemoveAspect {
        entity: "E".into(),
        aspect: "A".into(),
    };
    for (target, expected) in [
        (
            field,
            vec![0, 3, 0, 0, 0, 1, b'E', 0, 0, 0, 1, b'A', 0, 0, 0, 1, b'F'],
        ),
        (removal, vec![0, 8, 0, 0, 0, 1, b'E', 0, 0, 0, 1, b'A']),
    ] {
        let mut output = crate::binary_output::BinaryOutput::with_capacity(expected.len());
        write_program_target(&mut output, &target).unwrap();
        let bytes = output.into_bytes();
        assert_eq!(bytes, expected);
        assert_eq!(
            decode_program_target(&mut BinaryInput::new(&bytes)).unwrap(),
            target
        );
    }
}
