use super::digest_hash::digest_hash_parts;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug)]
struct AuthorityBackedDigestLabel {
    label: String,
    source_identity: Option<WorthQueryEvidenceIdentity>,
}

impl AuthorityBackedDigestLabel {
    fn comparison_key(&self) -> (&str, Option<&str>) {
        (
            &self.label,
            self.source_identity
                .as_ref()
                .map(WorthQueryEvidenceIdentity::terminal_projection_for_reporting),
        )
    }

    fn from_parts(parts: &[String]) -> Self {
        Self {
            label: digest_hash_parts(parts),
            source_identity: None,
        }
    }

    fn from_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> Self {
        Self {
            label: identity.terminal_projection_for_reporting().to_string(),
            source_identity: Some(identity.clone()),
        }
    }

    fn as_str(&self) -> &str {
        &self.label
    }

    fn evidence_identity(
        &self,
        scope: WorthQueryEvidenceScope,
        identity_family: &'static str,
        field_tag: &'static str,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        if let Some(source_identity) = &self.source_identity {
            return Ok(source_identity.clone());
        }
        Ok(worth_query_evidence_identity(scope)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                identity_family,
            )
            .field_value(WorthQueryEvidenceTag::new(field_tag), self.as_str())
            .seal()?)
    }
}

impl PartialEq for AuthorityBackedDigestLabel {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_key() == other.comparison_key()
    }
}

impl Eq for AuthorityBackedDigestLabel {}

impl PartialOrd for AuthorityBackedDigestLabel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AuthorityBackedDigestLabel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.comparison_key().cmp(&other.comparison_key())
    }
}

impl std::hash::Hash for AuthorityBackedDigestLabel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.comparison_key().hash(state);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlanDigest(AuthorityBackedDigestLabel);

impl PlanDigest {
    pub fn from_parts(parts: &[String]) -> Self {
        Self(AuthorityBackedDigestLabel::from_parts(parts))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.0.evidence_identity(
            WorthQueryEvidenceScope::MutationEvidenceSourceDigest,
            "execution_plan_digest_v1",
            "plan_digest",
        )?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BasisDigest(AuthorityBackedDigestLabel);

impl BasisDigest {
    #[cfg(test)]
    pub fn from_parts(parts: &[String]) -> Self {
        Self(AuthorityBackedDigestLabel::from_parts(parts))
    }

    pub fn from_evidence_identity(identity: &WorthQueryEvidenceIdentity) -> Self {
        Self(AuthorityBackedDigestLabel::from_evidence_identity(identity))
    }

    #[cfg(test)]
    pub fn from_collision_for_test(
        label: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self(AuthorityBackedDigestLabel {
            label: label.into(),
            source_identity: Some(source_identity),
        })
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.0.evidence_identity(
            WorthQueryEvidenceScope::BasisDigest,
            "basis_digest_evidence_v1",
            "basis_digest",
        )?)
    }
}

macro_rules! runtime_digest {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn from_parts(parts: &[String]) -> Self {
                Self(digest_hash_parts(parts))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

runtime_digest!(ResultDigest);
runtime_digest!(LineageDigest);
runtime_digest!(CorrespondenceOutcomeDigest);
runtime_digest!(CorrespondenceCostPostureDigest);
runtime_digest!(HistoricalPathClassDigest);
runtime_digest!(HistoricalCostPostureDigest);
runtime_digest!(FailureDigest);
runtime_digest!(CounterSnapshotDigest);
