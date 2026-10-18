//! Ordinary source and changed-output consumers, instantiated at each actual schema.
use super::super::schema::*;
use super::host::*;
use super::model::NestedValue;
use crate::document_retention_model::operator_identity::*;
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::*, primary_graph::*,
};
pub fn produce_changed<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
) -> WorthQueryApplicationOutputDemandSettlement<super::super::query::ValueQuery>
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world());
    let mut demand = request
        .demand(super::super::producer::Demand)
        .start_in_program::<_, super::super::program::Root>(host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the installed three-role producer must settle")
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    settled
}
pub fn read<P: ApplicationProgramDefinition<CanonicalSchema>>(host: &Host<P>) -> NestedValue
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(super::super::query::ReadValue {
            identity: "authored-nested".to_owned(),
        })
        .execute()
        .expect("ordinary authenticated query decodes the nested value")
        .rows()[0]
        .clone()
}

#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub fn read_derived<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
) -> super::super::output::DerivedValue
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(super::super::derived_query::ReadValue {
            identity: "derived-report".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}

pub fn source_id<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let request = request_scope();
    let runtime = host.runtime();
    runtime
        .with_application_advancement(&request, |phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    "authored-nested".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .expect("source identity resolves under the actual native request")
}

pub fn read_companion<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
    identity: &str,
) -> String
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(super::read::ReadValue {
            identity: identity.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}

#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub fn page<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
) -> WorthQueryApplicationContinuationPageResult<
    CanonicalSchema,
    super::super::page_query::PageQuery,
    super::super::query::Parameters,
    super::super::page_query::PageRow,
    Derived,
>
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(super::super::page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .page(
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap()
}
