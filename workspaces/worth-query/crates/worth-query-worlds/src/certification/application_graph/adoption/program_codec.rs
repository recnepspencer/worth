//! Stable, authority-free archive descriptions for evolving application programs.

pub use worth_query_host::facade::declaration::application_program::{
    ApplicationFeatureSpec, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity,
};
pub use worth_query_package_archive::facade::{
    decode_application_program_description, encode_application_program_description,
    WorthQueryApplicationProgramArchiveCompatibility, WorthQueryPackageArchiveDenialKind,
    WorthQueryPackageArchiveLimits,
};

pub use crate::facade::certification::document_retention_model::programs::{
    RetentionProgramP0, RetentionProgramP1,
};
pub use crate::facade::certification::document_retention_model::schema::DocumentRetentionSchema;

pub struct RetentionProgramP0IdentityWithP1Meaning;

impl ApplicationProgramDefinition<DocumentRetentionSchema>
    for RetentionProgramP0IdentityWithP1Meaning
{
    type Contributions = <RetentionProgramP1 as ApplicationProgramDefinition<
        DocumentRetentionSchema,
    >>::Contributions;
    type Outputs =
        <RetentionProgramP1 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP1 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;

    const IDENTITY: ApplicationProgramIdentity =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        RetentionProgramP1::feature_specs()
    }
}
