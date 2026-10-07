use sha2::{Digest, Sha256};
use worth_foundational::facade::CanonicalDigestId;

pub(crate) fn hash_parts(parts: &[String]) -> String {
    canonical_hash_parts(parts).render_hex()
}

pub(crate) fn canonical_hash_parts(parts: &[String]) -> CanonicalDigestId {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    CanonicalDigestId::new(hasher.finalize().into())
}

pub(crate) fn hash_parts_with_digests(parts: &[String], digests: &[&CanonicalDigestId]) -> String {
    canonical_hash_parts_with_digests(parts, digests).render_hex()
}

pub(crate) enum AdmittedHashStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

pub(crate) fn hash_parts_with_digests_admitted<Stop>(
    parts: &[String],
    digests: &[&CanonicalDigestId],
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<String, AdmittedHashStop<Stop>> {
    let part_count =
        u64::try_from(parts.len()).map_err(|_| AdmittedHashStop::AccountingOverflow)?;
    let digest_count =
        u64::try_from(digests.len()).map_err(|_| AdmittedHashStop::AccountingOverflow)?;
    let visits = part_count
        .checked_add(digest_count)
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    admit(visits, 0).map_err(AdmittedHashStop::Admission)?;
    let framed_parts = parts
        .iter()
        .try_fold(0_u64, |sum, part| {
            sum.checked_add(8)?
                .checked_add(u64::try_from(part.len()).ok()?)
        })
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    let framed_digests = digest_count
        .checked_mul(38)
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    let input_bytes = framed_parts
        .checked_add(framed_digests)
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    let blocks = input_bytes
        .checked_add(9)
        .and_then(|bytes| bytes.checked_add(63))
        .map(|bytes| bytes / 64)
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    let work = input_bytes
        .checked_add(blocks)
        // SHA finalization initializes one 32-byte digest. Rendering the
        // separate 64-byte hex output is accounted for below.
        .and_then(|work| work.checked_add(33))
        .and_then(|work| work.checked_add(64))
        .ok_or(AdmittedHashStop::AccountingOverflow)?;
    admit(work, 64).map_err(AdmittedHashStop::Admission)?;
    Ok(canonical_hash_parts_with_digests(parts, digests).render_hex())
}

fn canonical_hash_parts_with_digests(
    parts: &[String],
    digests: &[&CanonicalDigestId],
) -> CanonicalDigestId {
    let mut hasher = Sha256::new();
    for digest in digests {
        hasher.update(b"digest");
        hasher.update(digest.bytes());
    }
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    CanonicalDigestId::new(hasher.finalize().into())
}
