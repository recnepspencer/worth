use super::super::evidence::{CanonicalDigestInputEvidence, CanonicalDigestInputId};
use super::super::resource_admission::CanonicalResourceAdmission;
use super::domain_tokens::domain_material_token;
use super::token_writer::append_u32_text;
use super::writer::{CanonicalMaterialResult, CanonicalMaterialWriter};

pub(in crate::canonicalization::digest_slots) fn digest_input_id(
    evidence: &CanonicalDigestInputEvidence,
    admission: Option<&mut CanonicalResourceAdmission<'_>>,
) -> CanonicalMaterialResult<CanonicalDigestInputId> {
    let mut material = match admission {
        Some(admission) => CanonicalMaterialWriter::admitted(usize::MAX, admission),
        None => CanonicalMaterialWriter::bounded(usize::MAX),
    };
    match evidence {
        CanonicalDigestInputEvidence::SingleSequence(sequence) => {
            material.append("sequence:")?;
            material.append(domain_material_token(sequence.domain()))?;
            material.append(":")?;
            material.append(sequence.version().as_str())?;
            material.append(":")?;
            append_u32_text(&mut material, sequence.cost().entry_count())?;
        }
        CanonicalDigestInputEvidence::DomainBundle(bundle)
        | CanonicalDigestInputEvidence::ExportBundle(bundle) => {
            material.append(match evidence {
                CanonicalDigestInputEvidence::DomainBundle(_) => "domain-bundle:",
                _ => "export-bundle:",
            })?;
            material.append(bundle.version().as_str())?;
            material.append(":")?;
            for (ordinal, sequence) in bundle.sequences().iter().enumerate() {
                if ordinal != 0 {
                    material.append(",")?;
                }
                material.append(domain_material_token(sequence.domain()))?;
            }
        }
    }
    Ok(CanonicalDigestInputId::new(material.finish_string()))
}
