//! One opted-in family with legitimate, data-grown source parameter routes.
use super::*;
type SourceQuery = super::super::route_query::RouteQuery;
const TIER: u8 = 2;
const PROFILE: &str = "parameter-route";
const OWNERSHIP: ProducerPublicationOwnership = ProducerPublicationOwnership::SourceScopeCascade;
fn output_key(row: &Row) -> String {
    format!("parameter:{}:{}", row.key, row.width)
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = super::super::route_query::Binding;
    type Entity = Realization;
    const IDENTITY: &'static str = PROFILE;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Initial),
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Preserve),
    ];
    fn profile_kind(_: &Row) -> &'static str {
        PROFILE
    }
}
#[derive(Clone)]
pub struct Demand(pub String, pub u64);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> super::super::route_query::ReadRoute {
        super::super::route_query::ReadRoute {
            key: self.0.clone(),
            partition: self.1,
        }
    }
}
split_phase!(
    initial,
    ParameterRouteInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "dispatch.parameter.initial"
);
split_phase!(
    preserve,
    ParameterRoutePreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "dispatch.parameter.preserve"
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    preserve::declare(initial::declare(schema))
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<initial::Producer>()?;
    contracts.producer::<preserve::Producer>()?;
    contracts.conditional::<initial::Readiness>()?;
    contracts.conditional::<preserve::Readiness>()?;
    Ok(())
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler(
        selections.clone(),
        fault,
        child_width,
        read_predecessor_orbits,
    ))?;
    setup.handler::<preserve::Binding, _>(preserve::Handler(
        selections,
        fault,
        child_width,
        read_predecessor_orbits,
    ))?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}
