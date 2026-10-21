use super::*;

pub(in crate::domain_computation::primary_graph::application_attempt::dependency_identity) fn derive_identity(
    rule_version: &str,
    entries: Vec<CanonicalBasisEntry>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), ()> {
    let version = CanonicalizationRuleVersion::new(rule_version).ok_or(())?;
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .map_err(|_| ())?;
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .map_err(|_| ())?;
    let derived = canonicalization()
        .digest()
        .derive_with_admission(ready, &mut |_, _| Ok::<_, ()>(()))
        .map_err(|_| ())?;
    Ok((
        *derived.value().bytes(),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}
