use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_foundational::facade::{
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisReadyArtifact, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::canonical_basis_ready_sequence::canonical_basis_ready_sequence;

#[derive(Deserialize)]
struct NativeCanonicalBasis {
    version: String,
    domain: CanonicalBasisDomain,
    entries: Vec<CanonicalBasisEntry>,
}

pub fn serialize<S>(basis: &CanonicalBasisReadyArtifact, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct NativeCanonicalBasisRef<'a> {
        version: &'a str,
        domain: CanonicalBasisDomain,
        entries: &'a [CanonicalBasisEntry],
    }

    let ready = canonical_basis_ready_sequence(basis);
    NativeCanonicalBasisRef {
        version: ready.version().as_str(),
        domain: ready.domain(),
        entries: ready.entries(),
    }
    .serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<CanonicalBasisReadyArtifact, D::Error>
where
    D: Deserializer<'de>,
{
    let basis = NativeCanonicalBasis::deserialize(deserializer)?;
    let version = CanonicalizationRuleVersion::new(basis.version)
        .ok_or_else(|| serde::de::Error::custom("invalid diagnostic canonical basis version"))?;
    match prepare_canonical_basis_sequence(version, basis.domain, basis.entries) {
        TransitionOutcome::Success(ready) => Ok(ready),
        other => Err(serde::de::Error::custom(format!(
            "diagnostic canonical basis was not ready: {other:?}"
        ))),
    }
}
