use crate::physical_runtime::durability::{
    decode_retirement, payload_is_retirement, RetirementRecord,
};

use super::StoreRecoveryBindingSampleDenial;

pub(super) enum ClassifiedWalPayload<'payload> {
    Member {
        binding: &'payload [u8],
        redo: &'payload [u8],
    },
    Retirement(RetirementRecord),
}

pub(super) fn classify_wal_payload(
    payload: &[u8],
) -> Result<ClassifiedWalPayload<'_>, StoreRecoveryBindingSampleDenial> {
    if payload_is_retirement(payload) {
        return decode_retirement(payload)
            .map(ClassifiedWalPayload::Retirement)
            .ok_or(StoreRecoveryBindingSampleDenial::InvalidWalMember);
    }
    let (binding, redo) = decode_wal_member_payload(payload)?;
    Ok(ClassifiedWalPayload::Member { binding, redo })
}

pub(super) fn decode_wal_member_payload(
    mut payload: &[u8],
) -> Result<(&[u8], &[u8]), StoreRecoveryBindingSampleDenial> {
    let binding = take_field(&mut payload)?;
    let redo = take_field(&mut payload)?;
    if binding.is_empty() || redo.is_empty() || !payload.is_empty() {
        return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
    }
    Ok((binding, redo))
}

fn take_field<'payload>(
    payload: &mut &'payload [u8],
) -> Result<&'payload [u8], StoreRecoveryBindingSampleDenial> {
    let length = payload
        .get(..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(StoreRecoveryBindingSampleDenial::InvalidWalMember)?;
    let end = 8_usize
        .checked_add(length)
        .ok_or(StoreRecoveryBindingSampleDenial::InvalidWalMember)?;
    let field = payload
        .get(8..end)
        .ok_or(StoreRecoveryBindingSampleDenial::InvalidWalMember)?;
    *payload = payload
        .get(end..)
        .ok_or(StoreRecoveryBindingSampleDenial::InvalidWalMember)?;
    Ok(field)
}

#[cfg(test)]
mod tests {
    use crate::physical_runtime::durability::{encode_retirement, RetiredArtifact};

    use super::{classify_wal_payload, ClassifiedWalPayload, StoreRecoveryBindingSampleDenial};

    #[test]
    fn a_retirement_frame_is_not_a_mutation_member() {
        let payload = encode_retirement(
            RetiredArtifact::Segment {
                segment: 1,
                generation: 2,
            },
            false,
            4,
            16,
        );
        let ClassifiedWalPayload::Retirement(record) = classify_wal_payload(&payload).unwrap()
        else {
            panic!("retirement intent must stay out of redo admission");
        };
        assert_eq!(record.artifact.generation(), 2);
        assert_eq!(record.bytes, 16);
    }

    #[test]
    fn a_truncated_retirement_frame_is_rejected() {
        let mut payload = encode_retirement(
            RetiredArtifact::Segment {
                segment: 1,
                generation: 2,
            },
            false,
            4,
            16,
        );
        payload.pop();
        assert!(matches!(
            classify_wal_payload(&payload),
            Err(StoreRecoveryBindingSampleDenial::InvalidWalMember)
        ));
    }

    #[test]
    fn a_two_field_member_stays_a_member() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1_u64.to_le_bytes());
        payload.push(b'a');
        payload.extend_from_slice(&1_u64.to_le_bytes());
        payload.push(b'b');
        let ClassifiedWalPayload::Member { binding, redo } =
            classify_wal_payload(&payload).unwrap()
        else {
            panic!("mutation members remain mutation members");
        };
        assert_eq!(binding, b"a");
        assert_eq!(redo, b"b");
    }
}
