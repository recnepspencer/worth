use worth_foundational::facade::CanonicalDigestDerivationDenial;

/// Refused canonical identity material, with the artifact's semantic subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeCausalDigestRefusal {
    pub subject: &'static str,
    pub cause: CanonicalDigestDerivationDenial,
}
