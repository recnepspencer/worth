//! Relational installation of the same authored retention program contracts.
use super::*;
use document::schema::DocumentRetentionSchema as Schema;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_installation::WorthQueryApplicationProgramRoster,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
pub(super) struct Program;
pub(super) struct Target;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (document::host::RelationalRetentionContribution,);
    type Outputs =
        <document::programs::RetentionProgramP0 as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules =
        <document::programs::RetentionProgramP0 as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home.relational-retention.p0.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        document::programs::RetentionProgramP0::feature_specs()
    }
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<Schema>>(
        visitor: V,
    ) -> V::Output {
        document::programs::RetentionProgramP0::adoption_authorization(visitor)
    }
}
impl ApplicationProgramDefinition<Schema> for Target {
    type Contributions = (document::host::RelationalRetentionContribution,);
    type Outputs =
        <document::programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules =
        <document::programs::RetentionProgramP1 as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home.relational-retention.p1.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        document::programs::RetentionProgramP1::feature_specs()
    }
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<Schema>>(
        visitor: V,
    ) -> V::Output {
        document::programs::RetentionProgramP1::adoption_authorization(visitor)
    }
}
pub(super) type Home = worth_query_host::facade::WorthQueryApplicationHome<Schema, Program>;
pub(super) fn open(
    location: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Home {
    let declaration = Schema::declaration().unwrap();
    let initial = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let target = ApplicationProgramAuthoring::<Schema, Target>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let roster = document::host::with_lifecycle_programs(
        WorthQueryApplicationProgramRoster::new().support(target),
        initial.identity(),
        initial.revision(),
    );
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        initial,
        roster,
        declaration, ((),), ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        document::host::seed_host,
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("actual relational Home: {cause:?}"))
}
