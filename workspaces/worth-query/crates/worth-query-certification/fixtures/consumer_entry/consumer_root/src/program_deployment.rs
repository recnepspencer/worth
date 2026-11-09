//! Explicit program-revision deployment grants, governed by Query capabilities.
#[path = "program_deployment/records.rs"]
mod storage_model;
use crate::ConsumerSchema;
pub use storage_model::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_topology_entry::Principal;
mod capability;
mod grant_seed;
mod installation;
pub use capability::*;

pub(crate) use grant_seed::{bind_scope, ConsumerDeploymentGrantSeed};

pub struct ConsumerProgramDeployment;
impl worth_query_decl::facade::application_schema::ApplicationSchemaContribution<ConsumerSchema>
    for ConsumerProgramDeployment
{
    const IDENTITY:
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity =
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity::new(
            "worth.query.certification.consumer-deployment.v1",
        );
    type Members = (DeploymentRecords, ConsumerProgramDeploymentContracts);
}
impl ConsumerProgramDeployment {
    pub fn reference(
    ) -> worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef<
        ConsumerSchema,
        Self,
    > {
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct ConsumerProgramDeploymentContracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<ConsumerSchema>
    for ConsumerProgramDeploymentContracts
{
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
            ConsumerSchema,
        >,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
            ConsumerSchema,
        >,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(installation::install(schema))
    }
}

impl
    worth_query_host::facade::application_contribution::WorthQueryApplicationContribution<
        ConsumerSchema,
    > for ConsumerProgramDeployment
{
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, ConsumerSchema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        Ok(())
    }
}
