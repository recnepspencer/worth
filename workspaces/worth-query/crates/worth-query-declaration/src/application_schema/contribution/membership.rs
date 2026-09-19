use std::collections::BTreeSet;

use super::{ApplicationSchemaContribution, ApplicationSchemaContributionIdentity};
use crate::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaContributionDenial {
    InvalidIdentity,
    DuplicateIdentity,
}

impl std::fmt::Display for ApplicationSchemaContributionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application schema contribution denied: {self:?}"
        )
    }
}

impl std::error::Error for ApplicationSchemaContributionDenial {}

impl From<ApplicationSchemaContributionDenial> for ApplicationSchemaDeclarationDenial {
    fn from(denial: ApplicationSchemaContributionDenial) -> Self {
        match denial {
            ApplicationSchemaContributionDenial::InvalidIdentity => {
                Self::InvalidContributionIdentity
            }
            ApplicationSchemaContributionDenial::DuplicateIdentity => {
                Self::DuplicateContributionIdentity
            }
        }
    }
}

/// Root-owned contribution membership accumulated before one canonical build.
#[derive(Debug)]
pub struct ApplicationSchemaContributionMembership<Schema> {
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
    identities: BTreeSet<ApplicationSchemaContributionIdentity>,
}

impl<Schema> ApplicationSchemaContributionMembership<Schema>
where
    Schema: ApplicationSchema,
{
    /// Registers one contribution whose schema marker exactly matches the root.
    ///
    /// A contribution from another schema cannot cross this boundary:
    ///
    /// ```compile_fail
    /// use worth_query_declaration::{
    ///     worth_query_application_contribution, worth_query_application_schema,
    /// };
    /// use worth_query_declaration::facade::application_schema::{
    ///     ApplicationSchemaContributionAuthoring, ApplicationSchemaDeclarationBuilder,
    /// };
    ///
    /// worth_query_application_schema! {
    ///     schema LeftSchema {
    ///         owner: "worth.example.left",
    ///         version: (1, 0),
    ///         members: |schema| { schema }
    ///     }
    /// }
    /// worth_query_application_schema! {
    ///     schema RightSchema {
    ///         owner: "worth.example.right",
    ///         version: (1, 0),
    ///         members: |schema| { schema }
    ///     }
    /// }
    /// worth_query_application_contribution! {
    ///     contribution RightContribution in RightSchema {
    ///         identity: "worth.example.right.contribution.v1",
    ///         members: |schema| { schema }
    ///     }
    /// }
    ///
    /// let left = ApplicationSchemaDeclarationBuilder::<LeftSchema>::for_schema()
    ///     .contributions();
    /// let _ = left.register::<RightContribution>();
    /// ```
    pub fn register<Contribution>(mut self) -> Result<Self, ApplicationSchemaContributionDenial>
    where
        Contribution: ApplicationSchemaContribution<Schema>,
    {
        let identity = <Contribution as ApplicationSchemaContribution<Schema>>::IDENTITY.clone();
        if !identity.is_valid() {
            return Err(ApplicationSchemaContributionDenial::InvalidIdentity);
        }
        if !self.identities.insert(identity.clone()) {
            return Err(ApplicationSchemaContributionDenial::DuplicateIdentity);
        }
        let first_member = self.builder.contribution_member_count();
        self.builder =
            <Contribution as ApplicationSchemaContribution<Schema>>::register_members(self.builder);
        self.builder
            .retain_contribution_closure(identity, first_member);
        Ok(self)
    }

    pub fn build(
        self,
    ) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
        self.builder.build()
    }
}

/// Begins explicit contribution composition on an existing root builder.
pub trait ApplicationSchemaContributionAuthoring<Schema> {
    fn contributions(self) -> ApplicationSchemaContributionMembership<Schema>;
}

impl<Schema> ApplicationSchemaContributionAuthoring<Schema>
    for ApplicationSchemaDeclarationBuilder<Schema>
where
    Schema: ApplicationSchema,
{
    fn contributions(self) -> ApplicationSchemaContributionMembership<Schema> {
        ApplicationSchemaContributionMembership {
            builder: self,
            identities: BTreeSet::new(),
        }
    }
}
