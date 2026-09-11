mod closure;
mod definition;
mod membership;
mod provenance;
mod schema_binding;

pub(in crate::application_schema) use closure::{
    lower_authored_contributions, validate_canonical_contributions,
    AuthoredApplicationSchemaContribution,
};

#[cfg(test)]
mod tests;

pub use definition::{ApplicationSchemaContributionIdentity, ApplicationSchemaContributionRef};
pub use membership::{
    ApplicationSchemaContributionAuthoring, ApplicationSchemaContributionDenial,
    ApplicationSchemaContributionMembership,
};
pub use provenance::ApplicationSchemaContributionProvenance;
pub use schema_binding::ApplicationSchemaContribution;
