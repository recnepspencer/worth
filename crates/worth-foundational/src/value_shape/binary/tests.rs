use super::*;

struct SignedByte;
impl BinaryValueDelegate for SignedByte {
    type Value = i8;
    const MAXIMUM_WIRE_BYTES: usize = 3;
    fn encode(value: &i8, output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        output.write(&[1, u8::from(*value < 0), value.unsigned_abs()])
    }
    fn admit(raw: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<i8, D> {
        let [version, sign, magnitude] = raw else {
            return Err(D::Malformed);
        };
        if *version != 1 {
            return Err(D::UnsupportedVersion {
                expected: 1,
                observed: u32::from(*version),
            });
        }
        if *sign > 1 {
            return Err(D::Malformed);
        }
        if *sign == 1 && *magnitude == 0 {
            return Err(D::NonCanonical);
        }
        admission.admit(2, 0)?;
        let value = if *sign == 1 {
            -i16::from(*magnitude)
        } else {
            i16::from(*magnitude)
        };
        i8::try_from(value).map_err(|_| D::Malformed)
    }
}

#[test]
fn literal_canonical_bytes_and_owner_refusals_are_exact() {
    // Literal wire oracle, independent of both owner codec functions.
    assert_eq!(
        encode_binary_value::<SignedByte>(&-7).unwrap().as_bytes(),
        &[1, 1, 7]
    );
    assert_eq!(decode_binary_value::<SignedByte>(&[1, 1, 7]), Ok(-7));
    assert_eq!(decode_binary_value::<SignedByte>(&[1, 1, 128]), Ok(i8::MIN));
    assert_eq!(
        decode_binary_value::<SignedByte>(&[1, 0, 128]),
        Err(D::Malformed)
    );
    assert_eq!(
        decode_binary_value::<SignedByte>(&[1, 2, 7]),
        Err(D::Malformed)
    );
    assert_eq!(
        decode_binary_value::<SignedByte>(&[1, 1, 0]),
        Err(D::NonCanonical)
    );
    assert_eq!(
        decode_binary_value::<SignedByte>(&[2, 0, 7]),
        Err(D::UnsupportedVersion {
            expected: 1,
            observed: 2
        })
    );
    for end in 0..3 {
        assert_eq!(
            decode_binary_value::<SignedByte>(&[1, 1, 7][..end]),
            Err(D::Malformed)
        );
    }
    let mut calls = 0;
    let result = decode_binary_value_with_admission::<SignedByte>(&[1, 0, 7, 0], &mut |_, _| {
        calls += 1;
        Ok(())
    });
    assert_eq!(result, Err(D::WireBytes));
    assert_eq!(calls, 0, "size refusal precedes owner/admission effects");
}

struct Corrupt;
impl BinaryValueDelegate for Corrupt {
    type Value = i8;
    const MAXIMUM_WIRE_BYTES: usize = 3;
    fn encode(value: &i8, output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        // Deliberately swallow even a comparison failure.
        let _ = SignedByte::encode(value, output);
        Ok(())
    }
    fn admit(raw: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<i8, D> {
        SignedByte::admit(raw, admission).map(|value| value + 1)
    }
}

#[test]
fn an_independent_byte_comparison_catches_a_corrupted_owner_result() {
    assert_eq!(
        decode_binary_value::<Corrupt>(&[1, 0, 7]),
        Err(D::InternalInconsistency)
    );
    assert_eq!(
        decode_binary_value_with_admission::<Corrupt>(&[1, 0, 7], &mut |_, _| Ok(())),
        Err(D::InternalInconsistency)
    );
}

struct Swallows;
impl BinaryValueDelegate for Swallows {
    type Value = ();
    const MAXIMUM_WIRE_BYTES: usize = 1;
    fn encode(_: &(), output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        let _ = output.write(&[1]);
        Ok(())
    }
    fn admit(_: &[u8], admission: &mut ValueDecodeAdmission<'_>) -> Result<(), D> {
        let _ = admission.admit(1, 0);
        Ok(())
    }
}

#[test]
fn first_original_refusal_wins_even_when_both_owner_functions_swallow_it() {
    // Three grants: platform input, owner construction, canonical comparison.
    for stop_at in 1..=3 {
        let mut calls = 0;
        let result = decode_binary_value_with_admission::<Swallows>(&[1], &mut |_, _| {
            calls += 1;
            if calls == stop_at {
                Err(D::Admission)
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(D::Admission));
        assert_eq!(
            calls, stop_at,
            "latched refusal never re-enters the original loan"
        );
    }
}

struct Overwrites;
impl BinaryValueDelegate for Overwrites {
    type Value = ();
    const MAXIMUM_WIRE_BYTES: usize = 1;
    fn encode(_: &(), output: &mut dyn BinaryValueWriter) -> Result<(), D> {
        let _ = output.write(&[1, 2]);
        let _ = output.write(&[1]);
        Ok(())
    }
    fn admit(_: &[u8], _: &mut ValueDecodeAdmission<'_>) -> Result<(), D> {
        Ok(())
    }
}

#[test]
fn an_encoder_cannot_clear_a_bound_refusal_or_return_partial_bytes() {
    assert_eq!(encode_binary_value::<Overwrites>(&()), Err(D::WireBytes));
}

#[test]
fn exactly_the_original_work_budget_at_the_wire_bound_is_required() {
    // Input3 + owner2 + comparison(3 + one chunk) = 9 work, no owned allocation.
    for budget in [8usize, 9] {
        let mut remaining = budget;
        let result =
            decode_binary_value_with_admission::<SignedByte>(&[1, 1, 7], &mut |work, bytes| {
                assert_eq!(bytes, 0);
                remaining = remaining.checked_sub(work).ok_or(D::Admission)?;
                Ok(())
            });
        assert_eq!(
            result,
            if budget == 9 {
                Ok(-7)
            } else {
                Err(D::Admission)
            }
        );
    }
}

struct InvalidBound<const MAXIMUM: usize>;
impl<const MAXIMUM: usize> BinaryValueDelegate for InvalidBound<MAXIMUM> {
    type Value = ();
    const MAXIMUM_WIRE_BYTES: usize = MAXIMUM;
    fn encode(_: &(), _: &mut dyn BinaryValueWriter) -> Result<(), D> {
        panic!("invalid descriptor reached owner")
    }
    fn admit(_: &[u8], _: &mut ValueDecodeAdmission<'_>) -> Result<(), D> {
        panic!("invalid descriptor reached owner")
    }
}

#[test]
fn a_bound_beyond_the_scalar_frame_refuses_before_owner_code() {
    type Wide = InvalidBound<{ crate::values::InlineBytes::MAXIMUM_LENGTH + 1 }>;
    assert_eq!(encode_binary_value::<Wide>(&()), Err(D::UnsupportedShape));
    assert_eq!(decode_binary_value::<Wide>(&[]), Err(D::UnsupportedShape));
    assert_eq!(
        encode_binary_value::<InvalidBound<0>>(&()),
        Err(D::UnsupportedShape)
    );
    assert_eq!(
        decode_binary_value::<InvalidBound<0>>(&[]),
        Err(D::UnsupportedShape)
    );
}
