//! A decision-input whose Initial creates one child record under its output
//! and whose Preserve drops every child it finds there (#277 step 1). A
//! Preserve successor round reaches the child its own chain deleted through
//! the `ChildOfFirst` link served at the chain head.
use super::super::super::schema::{
    Model as SourceEntity, ModelKey as SourceKey, ModelValue as SourceValue,
};
use super::*;
type Scope = ApplicationMutationFieldScope<SourceKey>;
fn source_key() -> SourceKey {
    SourceKey::at("ModelKey")
}
fn source_value() -> SourceValue {
    SourceValue::at("ModelValue")
}
const PROFILE: &str = "child-drop";
pub fn output_key(row: &query::Row) -> String {
    format!("first:{}", row.key)
}
pub fn child_key(row: &query::Row) -> String {
    format!("kid:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value
}
/// The children of one output, one member per source key.
pub struct Kids<C>(std::marker::PhantomData<fn() -> C>);
impl<C: ApplicationMutationOutputContract<Schema>> WorthQueryApplicationOutputRoleFamily
    for Kids<C>
{
    type Schema = Schema;
    type Contract = C;
    type Entity = ChildOutput;
    const PREFIX: &'static str = "kid.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 0;
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::producer_model::Binding;
    type Entity = FirstOutput;
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
    fn source_intent(&self) -> query::producer_model::Read {
        query::producer_model::Read::from(self.0.clone())
    }
}
phase!(
    initial,
    SourceScopeCascade,
    output,
    producer_model,
    ChildDropInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.child-drop.initial",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
phase!(
    preserve,
    SourceScopeCascade,
    output,
    producer_model,
    ChildDropPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.child-drop.preserve",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
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
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<initial::Binding, _>(initial::Handler { derive: derived })?;
    setup.handler::<preserve::Binding, _>(preserve::Handler { derive: derived })?;
    setup.producer::<initial::Producer>(initial::Provider)?;
    setup.producer::<preserve::Producer>(preserve::Provider)?;
    setup.conditional::<initial::Readiness>(())?;
    setup.conditional::<preserve::Readiness>(())
}

pub struct Feature;
pub struct InputPort;
pub struct OutputPort;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<InputPort, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "owned-outputs.child-drop.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "child-drop-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "child-drop-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
