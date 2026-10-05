mod algorithm;
mod domain_tokens;
mod input_id;
mod sequence;
mod sha256_hash;
mod token_writer;
mod value;
mod writer;

pub(crate) use sequence::basis_sequence_material;
pub(crate) use value::{struct_value_material, value_material};

pub(super) use input_id::digest_input_id;
pub(super) use sha256_hash::sha256_digest;

use algorithm::append_algorithm_material;
use sequence::{append_bundle_material, append_sequence_material};
use token_writer::append_token;

use super::algorithm::CanonicalDigestAlgorithmMetadata;
use super::evidence::CanonicalDigestInputEvidence;
use super::resource_admission::CanonicalResourceAdmission;
use writer::{CanonicalEncodedMaterial, CanonicalMaterialResult, CanonicalMaterialWriter};

pub(super) fn canonical_digest_material(
    algorithm: &CanonicalDigestAlgorithmMetadata,
    evidence: &CanonicalDigestInputEvidence,
    maximum_encoded_bytes: usize,
    admission: Option<&mut CanonicalResourceAdmission<'_>>,
) -> CanonicalMaterialResult<CanonicalEncodedMaterial> {
    let mut material = match admission {
        Some(admission) => CanonicalMaterialWriter::admitted(maximum_encoded_bytes, admission),
        None => CanonicalMaterialWriter::bounded(maximum_encoded_bytes),
    };
    append_algorithm_material(&mut material, algorithm)?;
    append_input_evidence_material(&mut material, evidence)?;
    Ok(material.finish())
}

fn append_input_evidence_material(
    material: &mut CanonicalMaterialWriter,
    evidence: &CanonicalDigestInputEvidence,
) -> CanonicalMaterialResult {
    match evidence {
        CanonicalDigestInputEvidence::SingleSequence(sequence) => {
            append_token(material, "input", "single")?;
            append_sequence_material(material, sequence)?;
        }
        CanonicalDigestInputEvidence::DomainBundle(bundle) => {
            append_token(material, "input", "domain-bundle")?;
            append_bundle_material(material, bundle)?;
        }
        CanonicalDigestInputEvidence::ExportBundle(bundle) => {
            append_token(material, "input", "export-bundle")?;
            append_bundle_material(material, bundle)?;
        }
    }
    Ok(())
}
