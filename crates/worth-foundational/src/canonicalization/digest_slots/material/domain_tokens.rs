use super::super::algorithm::CanonicalDigestInputShape;
use crate::canonicalization::CanonicalBasisDomain;

pub(super) fn input_shape_token(shape: CanonicalDigestInputShape) -> &'static str {
    match shape {
        CanonicalDigestInputShape::SingleSequence => "single-sequence",
        CanonicalDigestInputShape::DomainBundle => "domain-bundle",
        CanonicalDigestInputShape::ExportBundle => "export-bundle",
    }
}

pub(crate) fn domain_material_token(domain: CanonicalBasisDomain) -> &'static str {
    match domain {
        CanonicalBasisDomain::Value => "value",
        CanonicalBasisDomain::AspectContract => "aspect-contract",
        CanonicalBasisDomain::AspectMask => "aspect-mask",
        CanonicalBasisDomain::AuthoritativeState => "authoritative-state",
        CanonicalBasisDomain::AuthoritativePatch => "authoritative-patch",
        CanonicalBasisDomain::Identity => "identity",
        CanonicalBasisDomain::Locator => "locator",
        CanonicalBasisDomain::Profile => "profile",
        CanonicalBasisDomain::Performance => "performance",
        CanonicalBasisDomain::BoundaryArtifact => "boundary-artifact",
        CanonicalBasisDomain::Transition => "transition",
        CanonicalBasisDomain::Diagnostic => "diagnostic",
        CanonicalBasisDomain::CompatibilityLowering => "compatibility-lowering",
        CanonicalBasisDomain::Future(value) => value,
    }
}
