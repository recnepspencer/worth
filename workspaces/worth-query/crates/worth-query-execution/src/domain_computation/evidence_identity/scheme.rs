use worth_foundational::facade::CanonicalizationRuleVersion;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryEvidenceIdentityScheme {
    V1,
    V2,
}

impl WorthQueryEvidenceIdentityScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "worth.query.evidence-identity.v1",
            Self::V2 => "worth.query.evidence-identity.v2",
        }
    }

    pub(crate) fn canonicalization_rule_version(self) -> CanonicalizationRuleVersion {
        match self {
            Self::V1 => CanonicalizationRuleVersion::new(self.as_str())
                .expect("evidence identity scheme version must stay canonical"),
            Self::V2 => CanonicalizationRuleVersion::new(self.as_str())
                .expect("evidence identity scheme version must stay canonical"),
        }
    }
}
