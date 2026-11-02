//! Basis flags retain legacy bytes and admit the explicit current-only contract.
use super::*;
use crate::binary_output::BinaryOutput;

#[test]
fn basis_support_keeps_literal_legacy_bytes_and_current_only_round_trips() {
    for (value, expected) in [
        (
            ApplicationQueryBasisSupport::current_and_pinned(),
            [0, 1, 0, 1, 0, 0],
        ),
        (
            ApplicationQueryBasisSupport::current_and_pinned().with_preview(),
            [0, 1, 0, 1, 0, 1],
        ),
        (
            ApplicationQueryBasisSupport::current_only(),
            [0, 1, 0, 0, 0, 0],
        ),
        (
            ApplicationQueryBasisSupport::current_only().with_preview(),
            [0, 1, 0, 0, 0, 1],
        ),
    ] {
        let mut output = BinaryOutput::with_capacity(6);
        write_basis_support(&mut output, value).unwrap();
        assert_eq!(output.into_bytes(), expected);
        let mut input = BinaryInput::new(&expected);
        assert_eq!(decode_basis_support(&mut input).unwrap(), value);
        assert!(input.is_finished());
    }
    for invalid in [[0, 0, 0, 0, 0, 0], [0, 0, 0, 1, 0, 0]] {
        assert_eq!(
            decode_basis_support(&mut BinaryInput::new(&invalid))
                .unwrap_err()
                .kind(),
            Kind::InvalidRecordShape
        );
    }
    assert_eq!(
        decode_basis_support(&mut BinaryInput::new(&[0, 1, 0, 2, 0, 0]))
            .unwrap_err()
            .kind(),
        Kind::InvalidBooleanEncoding
    );
    assert_eq!(
        decode_basis_support(&mut BinaryInput::new(&[0, 1, 0, 0]))
            .unwrap_err()
            .kind(),
        Kind::Truncated
    );
}
