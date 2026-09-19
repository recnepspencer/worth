use super::ApplicationSchemaContributionIdentity;
use crate::application_schema::{ApplicationSchema, ApplicationSchemaDeclarationBuilder};

/// One explicitly registered set of members for a single application schema.
///
/// Implementations add typed members directly to the root declaration builder.
/// They cannot return an erased fragment or target a different schema.
pub trait ApplicationSchemaContribution<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    const IDENTITY: ApplicationSchemaContributionIdentity;

    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema>;
}
