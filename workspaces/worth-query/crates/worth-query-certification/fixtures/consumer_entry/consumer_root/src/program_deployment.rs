//! Explicit program-revision deployment grants, governed by Query capabilities.
use crate::ConsumerSchema;
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_relation,
};
use worth_query_topology_entry::Principal;
mod capability;
mod grant_seed;
mod installation;
pub use capability::*;

pub(crate) use grant_seed::{bind_scope, ConsumerDeploymentGrantSeed};
worth_query_entity!(pub ProgramDeploymentScope for ConsumerSchema);
worth_query_aspect!(pub ProgramDeploymentScopeFacts for ConsumerSchema, ProgramDeploymentScope; identity = AspectIdentity(0x91764101), revision = AspectContractRevision(1),);
worth_query_field!(pub ProgramDeploymentSourceRevision for ConsumerSchema, ProgramDeploymentScope, ProgramDeploymentScopeFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_entity!(pub ProgramDeploymentGrant for ConsumerSchema);
worth_query_aspect!(pub ProgramDeploymentGrantFacts for ConsumerSchema, ProgramDeploymentGrant; identity = AspectIdentity(0x91764102), revision = AspectContractRevision(1),);
worth_query_field!(pub ProgramDeploymentGrantIdentity for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantSourceRevision for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantAction for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantPurpose for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantStatus for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub ProgramDeploymentGrantNotBefore for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantNotAfter for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantDepth for ConsumerSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub ProgramDeploymentGrantResource in ConsumerSchema, ProgramDeploymentGrant => ProgramDeploymentScope; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantParent in ConsumerSchema, ProgramDeploymentGrant => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantGrantor in ConsumerSchema, Principal => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantGrantee in ConsumerSchema, Principal => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);

worth_query_decl::facade::worth_query_application_contribution! {
    pub contribution ConsumerProgramDeployment in ConsumerSchema {
        identity: "worth.query.certification.consumer-deployment.v1",
        members: |schema| { installation::install(schema) }
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
