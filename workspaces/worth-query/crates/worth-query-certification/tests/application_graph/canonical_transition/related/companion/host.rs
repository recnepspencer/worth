//! Ordinary installed hosts and producers used by the companion transition proof.
use super::{super::schema::*, model::NestedValue, producer};
use crate::document_retention_model::operator_identity::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_entry::*,
    application_installation::*,
    declaration::{application_program::*, authentication::*},
    primary_graph::*,
};
pub type Host<P> = WorthQueryProgramApplicationRuntime<CanonicalSchema, P>;
pub fn program<P: ApplicationProgramDefinition<CanonicalSchema>>(
) -> ValidatedApplicationProgram<CanonicalSchema, P>
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    ApplicationProgramAuthoring::<CanonicalSchema, P>::begin()
        .validated_program()
        .unwrap()
}
#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub fn install<
    P: ApplicationProgramDefinition<
        CanonicalSchema,
        Contributions = (CanonicalContribution,),
        Outputs = ApplicationProgramOutputs<(super::super::program::Root, super::program::Root)>,
    >,
>(
    value: NestedValue,
    roster: WorthQueryApplicationProgramRoster<'_, CanonicalSchema>,
) -> Result<Host<P>, WorthQueryInMemoryApplicationDenial>
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    in_memory_rostered_program(
        ApplicationProgramAuthoring::<CanonicalSchema, P>::begin()
            .validated_program()
            .unwrap(),
        roster,
        CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        super::super::host::limits(),
        |_, graph, installed| {
            let binding = installed
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("canonical-value-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    WorthQueryApplicationEntityKey::new("nested-row").unwrap(),
                )
                .field(
                    DocumentIdentityField::reference(),
                    "authored-nested".to_owned(),
                )
                .field(DocumentValueField::reference(), value),
            )?;
            Ok(())
        },
    )
}
pub fn produce<P: ApplicationProgramDefinition<CanonicalSchema>>(
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
        .demand(producer::Demand)
        .start_in_program::<_, super::program::Root>(host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the installed three-role producer must settle")
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    settled
}
#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub fn edit_source<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
    value: NestedValue,
    idempotency: u64,
) -> WorthQueryApplicationCommitReceipt
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(super::super::edit::Input {
            identity: "authored-nested".to_owned(),
            value,
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_performed::<P, super::super::program::Root>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the native program-owned source edit executes");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source edit must retain its required-output custody");
    };
    let controls = WorthQueryOutputDemandControls::new(
        std::num::NonZeroUsize::new(4096).unwrap(),
        std::num::NonZeroUsize::new(65_536).unwrap(),
    );
    let mut started = performed
        .start_required_outputs(host, &request, controls)
        .unwrap_or_else(|failure| {
            panic!(
                "the source edit must start its fixed required output: {:?}",
                failure.denial()
            )
        });
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(host, &request)
        .unwrap()
    else {
        panic!("the source edit's real required-output producer must settle");
    };
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    settled
        .root_receipt()
        .expect("the real regeneration has a producer receipt")
        .clone()
}

pub fn capture<P: ApplicationProgramDefinition<CanonicalSchema>>(
    host: &Host<P>,
) -> WorthQueryApplicationCheckpoint
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    host.runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap()
}
pub fn restore<
    P: ApplicationProgramDefinition<
        CanonicalSchema,
        Contributions = (CanonicalContribution,),
        Outputs = ApplicationProgramOutputs<(super::super::program::Root, super::program::Root)>,
    >,
>(
    checkpoint: WorthQueryApplicationCheckpoint,
    roster: WorthQueryApplicationProgramRoster<'_, CanonicalSchema>,
) -> Host<P>
where
    P::Outputs: ApplicationProgramOutputsShape<CanonicalSchema>,
{
    in_memory_rostered_program_from_checkpoint(
        program::<P>(),
        roster,
        CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        super::super::host::limits(),
        checkpoint,
    )
    .unwrap()
}
