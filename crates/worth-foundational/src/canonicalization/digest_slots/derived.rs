use super::admission::admitted_algorithm_clone;
use super::algorithm::CanonicalDigestAlgorithmMetadata;
use super::evidence::{CanonicalDigestDerivationInput, CanonicalDigestInputId};
use super::material::{digest_input_id, sha256_digest};
use super::resource_admission::{
    CanonicalDigestAdmissionStop, CanonicalDigestPreparationStop, CanonicalResourceAdmission,
};
use super::CanonicalDigestDerivationReadyArtifact;
use super::CanonicalDigestWorkEvidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDerivedDigest {
    metadata: CanonicalDigestMetadata,
    value: CanonicalDigestValue,
}

impl CanonicalDerivedDigest {
    fn new(metadata: CanonicalDigestMetadata, value: CanonicalDigestValue) -> Self {
        Self { metadata, value }
    }

    pub fn metadata(&self) -> &CanonicalDigestMetadata {
        &self.metadata
    }

    pub fn value(&self) -> &CanonicalDigestValue {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDigestMetadata {
    algorithm: CanonicalDigestAlgorithmMetadata,
    input_id: CanonicalDigestInputId,
    entry_count: u32,
    work: CanonicalDigestWorkEvidence,
}

impl CanonicalDigestMetadata {
    fn new(
        algorithm: CanonicalDigestAlgorithmMetadata,
        input_id: CanonicalDigestInputId,
        entry_count: u32,
        work: CanonicalDigestWorkEvidence,
    ) -> Self {
        Self {
            algorithm,
            input_id,
            entry_count,
            work,
        }
    }

    pub fn algorithm(&self) -> &CanonicalDigestAlgorithmMetadata {
        &self.algorithm
    }

    pub fn input_id(&self) -> &CanonicalDigestInputId {
        &self.input_id
    }

    pub const fn entry_count(&self) -> u32 {
        self.entry_count
    }

    pub const fn work(&self) -> CanonicalDigestWorkEvidence {
        self.work
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDigestValue {
    bytes: [u8; 32],
}

impl CanonicalDigestValue {
    fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalDigestDerivationDenial {
    UnsupportedAlgorithm,
    RuleVersionMismatch,
    InputDomainMismatch,
    InputShapeMismatch,
    EntryLimitExceeded { maximum: u32, actual: u32 },
    EncodedByteLimitExceeded { maximum: usize, attempted: usize },
}

pub fn derive_canonical_digest(
    ready: CanonicalDigestDerivationReadyArtifact,
) -> CanonicalDerivedDigest {
    derive_canonical_digest_admitted(ready, None)
        .expect("ordinary digest derivation has no resource-refusal port")
}

pub fn derive_canonical_digest_with_admission<Stop>(
    ready: CanonicalDigestDerivationReadyArtifact,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<CanonicalDerivedDigest, CanonicalDigestAdmissionStop<Stop>> {
    let mut refusal = None;
    let mut admission = |work, bytes| {
        admit(work, bytes).map_err(|stop| {
            refusal = Some(stop);
        })
    };
    let result = derive_canonical_digest_admitted(ready, Some(&mut admission));
    result.map_err(|stop| stop.preserve(refusal))
}

fn derive_canonical_digest_admitted(
    ready: CanonicalDigestDerivationReadyArtifact,
    mut admission: Option<&mut CanonicalResourceAdmission<'_>>,
) -> Result<CanonicalDerivedDigest, CanonicalDigestPreparationStop> {
    let (input, _proofs, _basis) = ready.into_parts().into_parts();
    debug_assert!(input.algorithm().id().is_sha256());
    let algorithm = admitted_algorithm_clone(input.algorithm(), admission.as_deref_mut())?;
    let input_id = digest_input_id(input.evidence(), admission.as_deref_mut())?;
    if let Some(admission) = admission {
        let work = input
            .work()
            .sha256_input_bytes()
            .checked_add(input.work().sha256_compression_block_count())
            .and_then(|work| work.checked_add(32 + 1))
            .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
        admission(work, 0).map_err(|()| CanonicalDigestPreparationStop::ResourceRefused)?;
    }
    let value = CanonicalDigestValue::new(sha256_digest(input.material()));
    let metadata = CanonicalDigestMetadata::new(
        algorithm,
        input_id,
        input.work().canonical_entry_count(),
        input.work(),
    );

    Ok(CanonicalDerivedDigest::new(metadata, value))
}

#[allow(dead_code)]
fn _input_type_is_owned(_: &CanonicalDigestDerivationInput) {}
