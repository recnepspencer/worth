use super::*;

pub(super) fn derive_identity(
    rule_version: &str,
    entries: Vec<CanonicalBasisEntry>,
    maximum_canonical_bytes: usize,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), ()> {
    let version = CanonicalizationRuleVersion::new(rule_version).ok_or(())?;
    let maximum_entries = u32::try_from(entries.len()).map_err(|_| ())?;
    let budget =
        CanonicalDigestWorkBudget::new(maximum_entries, maximum_canonical_bytes).ok_or(())?;
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .map_err(|_| ())?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .map_err(|_| ())?;
    let derived = canonicalization().digest().derive(ready);
    Ok((
        *derived.value().bytes(),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}
