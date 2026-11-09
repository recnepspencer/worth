//! A producer keeps only its source parity plus an authored member it reads.
use super::super::schema::{
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
const PROFILE: &str = "parity-input";
pub fn output_key(row: &query::Row) -> String {
    format!("parity:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value % 2
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
    ParityInputInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.parity-input.initial",
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
    ParityInputPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.parity-input.preserve",
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
    const IDENTITY: &'static str = "owned-outputs.parity-input.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "parity-input-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "parity-input-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
