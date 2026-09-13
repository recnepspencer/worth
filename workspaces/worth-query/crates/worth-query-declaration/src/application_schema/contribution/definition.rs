use std::borrow::Cow;
use std::marker::PhantomData;

use super::ApplicationSchemaContribution;
use crate::application_schema::ApplicationSchema;

/// Stable declaration identity for one application contribution.
///
/// The identity names composition ownership. Contribution members still enter
/// the root schema's ordinary canonical declaration and remain its authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationSchemaContributionIdentity(Cow<'static, str>);

impl ApplicationSchemaContributionIdentity {
    pub const fn new(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    pub fn from_untrusted(identity: String) -> Self {
        Self(Cow::Owned(identity))
    }

    pub const fn as_str(&self) -> &str {
        match &self.0 {
            Cow::Borrowed(identity) => identity,
            Cow::Owned(identity) => identity.as_str(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.as_str().is_empty()
            && self.as_str().trim() == self.as_str()
            && !self
                .as_str()
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
    }
}

/// Typed reference proving that a contribution belongs to `Schema`.
pub struct ApplicationSchemaContributionRef<Schema, Contribution> {
    _marker: PhantomData<fn() -> (Schema, Contribution)>,
}

impl<Schema, Contribution> ApplicationSchemaContributionRef<Schema, Contribution>
where
    Schema: ApplicationSchema,
    Contribution: ApplicationSchemaContribution<Schema>,
{
    #[doc(hidden)]
    pub const fn from_contribution() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn identity(&self) -> ApplicationSchemaContributionIdentity {
        <Contribution as ApplicationSchemaContribution<Schema>>::IDENTITY.clone()
    }
}

impl<Schema, Contribution> Copy for ApplicationSchemaContributionRef<Schema, Contribution> {}

impl<Schema, Contribution> Clone for ApplicationSchemaContributionRef<Schema, Contribution> {
    fn clone(&self) -> Self {
        *self
    }
}
