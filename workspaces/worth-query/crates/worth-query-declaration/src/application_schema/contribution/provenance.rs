use super::ApplicationSchemaContributionIdentity;

/// Canonical ownership of schema members by one explicit contribution.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationSchemaContributionProvenance {
    identity: ApplicationSchemaContributionIdentity,
    member_ordinals: Vec<u32>,
}

impl ApplicationSchemaContributionProvenance {
    pub(super) fn from_canonical_parts(
        identity: ApplicationSchemaContributionIdentity,
        member_ordinals: Vec<u32>,
    ) -> Self {
        Self {
            identity,
            member_ordinals,
        }
    }

    pub fn from_untrusted_parts(identity: String, member_ordinals: Vec<u32>) -> Self {
        Self {
            identity: ApplicationSchemaContributionIdentity::from_untrusted(identity),
            member_ordinals,
        }
    }

    pub const fn identity(&self) -> &ApplicationSchemaContributionIdentity {
        &self.identity
    }

    pub fn member_ordinals(&self) -> &[u32] {
        &self.member_ordinals
    }
}
