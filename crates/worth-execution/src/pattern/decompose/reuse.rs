use worth_foundational::PartitionIdentity;

use crate::oracle::CanonicalBits;

use super::{DecomposeInputDenial, DecomposeReuse};

pub(super) fn changed_back_identities<I: CanonicalBits, B: CanonicalBits>(
    identities: &[PartitionIdentity],
    previous: Option<(&[I], &[B])>,
    interiors: &[I],
    slices: &[B],
    reuse: &mut DecomposeReuse,
    max_encoding_bytes: u64,
) -> Result<Vec<PartitionIdentity>, DecomposeInputDenial> {
    let mut changed = Vec::new();
    for (index, identity) in identities.iter().copied().enumerate() {
        let same = if let Some((old_interiors, old_slices)) = previous {
            same_encoding(&old_interiors[index], &interiors[index], max_encoding_bytes)?
                && same_encoding(&old_slices[index], &slices[index], max_encoding_bytes)?
        } else {
            false
        };
        if same {
            reuse.back_substitutions_reused += 1;
        } else {
            changed.push(identity);
        }
    }
    Ok(changed)
}

pub(super) fn same_encoding<T: CanonicalBits>(
    left: &T,
    right: &T,
    max_encoding_bytes: u64,
) -> Result<bool, DecomposeInputDenial> {
    let length = left
        .canonical_len()
        .ok_or(DecomposeInputDenial::InvalidCanonicalEncoding)?;
    if u64::try_from(length)
        .ok()
        .is_none_or(|length| length > max_encoding_bytes)
    {
        return Err(DecomposeInputDenial::StagedCapacityExceeded);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| DecomposeInputDenial::InvalidCanonicalEncoding)?;
    let mut valid = true;
    let complete = left.visit_canonical_bits(&mut |chunk| {
        if chunk.len() > length.saturating_sub(bytes.len()) {
            valid = false;
            return false;
        }
        bytes.extend_from_slice(chunk);
        true
    });
    if !complete || !valid || bytes.len() != length {
        return Err(DecomposeInputDenial::InvalidCanonicalEncoding);
    }
    let right_length = right
        .canonical_len()
        .ok_or(DecomposeInputDenial::InvalidCanonicalEncoding)?;
    if u64::try_from(right_length)
        .ok()
        .is_none_or(|length| length > max_encoding_bytes)
    {
        return Err(DecomposeInputDenial::StagedCapacityExceeded);
    }
    if right_length != length {
        return Ok(false);
    }
    let mut position: usize = 0;
    let mut malformed = false;
    let complete = right.visit_canonical_bits(&mut |chunk| {
        if chunk.len() > length.saturating_sub(position) {
            malformed = true;
            return false;
        }
        let end = position + chunk.len();
        if bytes.get(position..end) != Some(chunk) {
            valid = false;
            return false;
        }
        position = end;
        true
    });
    if malformed || (!complete && valid) || (complete && position != length) {
        return Err(DecomposeInputDenial::InvalidCanonicalEncoding);
    }
    Ok(valid && complete && position == length)
}
