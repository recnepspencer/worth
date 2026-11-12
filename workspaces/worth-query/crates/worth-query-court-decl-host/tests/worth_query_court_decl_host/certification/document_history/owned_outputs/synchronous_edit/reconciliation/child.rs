//! A separate installed family derives children from tracked produced positions.
use super::super::super::query::second_output as source;
use super::*;
type SourceEntity = SecondOutput;
type SourceKey = SecondKey;
type Scope = ApplicationMutationFieldScope<SecondKey>;
const PROFILE: &str = "reconciled-child";
fn source_key() -> SecondKey {
    SecondKey::at("SecondKey")
}
fn source_value() -> SecondValue {
    SecondValue::at("SecondValue")
}
fn output_key(row: &query::Row) -> String {
    format!("reconciled-child:{}", row.key)
}
#[derive(Clone, Debug)]
pub enum Denial {
    InitialHasPriorOutput,
    PreserveMissingOutput,
    DerivedValueOutOfRange,
    MalformedSourceGraph,
}
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "reconciled-child.denial" });
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = source::Binding;
    type Entity = ChildOutput;
    const IDENTITY: &'static str = PROFILE;
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Initial),
        WorthQueryProducerApplicability::new(PROFILE, WorthQueryProducerLifecyclePosture::Preserve),
    ];
    fn profile_kind(_: &query::Row) -> &'static str {
        PROFILE
    }
}
#[derive(Clone)]
pub struct Demand(pub String);
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> source::Read {
        source::Read(self.0.clone())
    }
}
reconciliation_child_phase!(
    initial,
    SourceScopeCascade,
    ReconciledChildInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.reconciled-child.initial",
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOfFirst
);
reconciliation_child_phase!(
    preserve,
    SourceScopeCascade,
    ReconciledChildPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.reconciled-child.preserve",
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOfFirst
);
pub(super) fn index_key(source: &str, value: u64) -> String {
    format!("reconciled-index:{source}:{value}")
}
pub struct Indices<C>(std::marker::PhantomData<fn() -> C>);
impl<C: ApplicationMutationOutputContract<Schema>> WorthQueryApplicationOutputRoleFamily
    for Indices<C>
{
    type Schema = Schema;
    type Contract = C;
    type Entity = ChildOutput;
    const PREFIX: &'static str = "index.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 0;
}
pub type Roles = WorthQueryRequiredFeaturePair<
    Schema,
    program::Child,
    initial::Producer,
    initial::Readiness,
    preserve::Producer,
    preserve::Readiness,
>;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    preserve::declare(initial::declare(schema))
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.feature::<Roles>()?;
    Ok(())
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    indices: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler {
        derive: |v| v * 100,
        indices,
    })?;
    setup.handler::<preserve::Binding, _>(preserve::Handler {
        derive: |v| v * 100,
        indices,
    })?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}
