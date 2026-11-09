use super::ApplicationSchemaContributionIdentity;
use crate::application_schema::ApplicationSchema;

/// A root-specific contribution composes reusable fragments and existing registrations.
/// Storage belongs to its fragment members; Host retains configuration ownership.
pub trait ApplicationSchemaContribution<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    const IDENTITY: ApplicationSchemaContributionIdentity;
    type Members;
}
