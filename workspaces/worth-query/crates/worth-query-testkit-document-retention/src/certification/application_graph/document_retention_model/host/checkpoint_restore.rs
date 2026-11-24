//! Opening and reopening document-retention Directory Homes under explicit rosters.
use crate::facade::directory_home::{with_directory_bootstrap, DirectoryHomeGuard};
use std::path::PathBuf;
pub use worth_query_host::facade::application_installation::{
    ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ApplicationWorkflowSpec,
    ValidatedApplicationProgram,
};
pub use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeRequest,
};

pub use super::super::{
    programs::{
        validated_changed_feature_program, validated_changed_operation_program,
        validated_first_program, validated_removed_assessment_supplier_program,
        validated_removed_operation_program, validated_second_program, RetentionProgramP0,
    },
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
pub use super::{host_limits, DocumentWorkflowDirectoryHome};

pub fn default_first_program_roster(
) -> WorthQueryApplicationProgramRoster<'static, DocumentRetentionSchema> {
    WorthQueryApplicationProgramRoster::new()
        .support(validated_second_program())
        .support(validated_changed_feature_program())
        .support(validated_changed_operation_program())
        .support(validated_removed_assessment_supplier_program())
        .support(validated_removed_operation_program())
}

pub fn open_directory_on_first_program(
    directory: PathBuf,
) -> DirectoryHomeGuard<DocumentRetentionSchema, RetentionProgramP0> {
    open_directory_with_limits(
        directory,
        validated_first_program(),
        default_first_program_roster(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
    )
    .unwrap_or_else(|refused| panic!("document-retention P0 directory open refused: {refused:?}"))
}

pub fn open_directory<Initial>(
    directory: PathBuf,
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
) -> Result<
    DirectoryHomeGuard<DocumentRetentionSchema, Initial>,
    WorthQueryApplicationHomeOpenRefused<DocumentRetentionSchema, Initial>,
>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs: ApplicationProgramOutputsShape<DocumentRetentionSchema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
            DocumentRetentionSchema,
        >,
{
    open_directory_with_limits(
        directory,
        initial,
        roster,
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
    )
}

pub fn open_directory_with_limits<Initial>(
    directory: PathBuf,
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
    limits: WorthQueryApplicationLimits,
) -> Result<
    DirectoryHomeGuard<DocumentRetentionSchema, Initial>,
    WorthQueryApplicationHomeOpenRefused<DocumentRetentionSchema, Initial>,
>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs: ApplicationProgramOutputsShape<DocumentRetentionSchema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
            DocumentRetentionSchema,
        >,
{
    open_directory_with_build(
        directory,
        initial,
        roster,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits,
    )
}

pub fn open_directory_with_build<Initial>(
    directory: PathBuf,
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
    build: ApplicationBuildIdentity,
    limits: WorthQueryApplicationLimits,
) -> Result<
    DirectoryHomeGuard<DocumentRetentionSchema, Initial>,
    WorthQueryApplicationHomeOpenRefused<DocumentRetentionSchema, Initial>,
>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs: ApplicationProgramOutputsShape<DocumentRetentionSchema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
            DocumentRetentionSchema,
        >,
{
    let home = open_raw_directory(directory.clone(), initial, roster, build, limits)?;
    Ok(DirectoryHomeGuard::wrap(home, directory))
}

pub fn open_raw_directory<Initial>(
    directory: PathBuf,
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
    build: ApplicationBuildIdentity,
    limits: WorthQueryApplicationLimits,
) -> Result<
    WorthQueryApplicationHome<DocumentRetentionSchema, Initial>,
    WorthQueryApplicationHomeOpenRefused<DocumentRetentionSchema, Initial>,
>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs: ApplicationProgramOutputsShape<DocumentRetentionSchema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
            DocumentRetentionSchema,
        >,
{
    let roster = super::with_lifecycle_programs(roster, initial.identity(), initial.revision());
    with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory),
            initial,
            roster,
            DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
            ((),),
            build,
            limits,
            super::seed_host,
        )
        .open_in_advancement(phase)
    })
}

pub fn open_workflow_directory_for<Initial, Spec>(
    directory: PathBuf,
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
    limits: WorthQueryApplicationLimits,
    resources: worth_query_host::facade::domain::WorthQueryApplicationWorkflowResourceCeiling,
) -> DocumentWorkflowDirectoryHome<Initial, Spec>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs: ApplicationProgramOutputsShape<DocumentRetentionSchema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
            DocumentRetentionSchema,
        >,
    Spec: ApplicationWorkflowSpec<Schema = DocumentRetentionSchema>,
{
    let home = open_raw_directory(
        directory.clone(),
        initial,
        roster,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits,
    )
    .unwrap_or_else(|refused| panic!("workflow directory Home open refused: {refused:?}"));
    let workflow = super::super::workflow::runtime::install_workflow_spec_for::<Initial, Spec>(
        home.runtime().runtime().installed_schema(),
        home.runtime().installed_program(),
        resources,
    );
    let (authentication, authentication_clock) =
        super::super::workflow::install_certification_authentication_with_clock(
            home.runtime().runtime().installed_schema(),
        );
    let workflow = home
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .expect("the workflow vocabulary belongs to the Directory Home");
    DocumentWorkflowDirectoryHome::wrap(workflow, authentication, authentication_clock, directory)
}
