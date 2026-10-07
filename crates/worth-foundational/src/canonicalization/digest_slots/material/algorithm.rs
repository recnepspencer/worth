use super::super::algorithm::CanonicalDigestAlgorithmMetadata;
use super::super::algorithm::CanonicalDigestInputDomain;
use super::domain_tokens::{domain_material_token, input_shape_token};
use super::token_writer::{append_token, append_token_parts};
use super::writer::{CanonicalMaterialResult, CanonicalMaterialWriter};

pub(super) fn append_algorithm_material(
    material: &mut CanonicalMaterialWriter,
    algorithm: &CanonicalDigestAlgorithmMetadata,
) -> CanonicalMaterialResult {
    append_token(material, "algorithm", algorithm.id().as_str())?;
    append_token(material, "version", algorithm.rule_version().as_str())?;
    append_token(
        material,
        "shape",
        input_shape_token(algorithm.input_shape()),
    )?;
    match algorithm.input_domain() {
        CanonicalDigestInputDomain::Single(domain) => append_token_parts(
            material,
            "domain",
            "",
            &["single:", domain_material_token(domain)],
        ),
        CanonicalDigestInputDomain::DomainBundle => {
            append_token(material, "domain", "domain-bundle")
        }
        CanonicalDigestInputDomain::ExportBundle => {
            append_token(material, "domain", "export-bundle")
        }
    }
}
