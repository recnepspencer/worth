//! A producer whose source is member `m` (read through the public member
//! source query) and which writes `m` itself: m = source + 1. Its decision
//! also reads `first:a` (PR C C3b entity PriorEffectRead fixture).
use super::super::schema::{
    Member as SourceEntity, MemberKey as SourceKey, MemberValue as SourceValue,
};
use super::*;
type Scope = ApplicationMutationFieldScope<SourceKey>;
fn source_key() -> SourceKey {
    SourceKey::at("MemberKey")
}
fn source_value() -> SourceValue {
    SourceValue::at("MemberValue")
}
const PROFILE: &str = "self-write";
pub fn output_key(row: &query::Row) -> String {
    format!("self:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::member_public::Binding;
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
    fn source_intent(&self) -> query::member_public::Read {
        query::member_public::Read::from(self.0.clone())
    }
}
phase!(
    initial,
    SourceScopeCascade,
    none,
    member_public,
    SelfWriteInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.self-write.initial",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
phase!(
    preserve,
    SourceScopeCascade,
    none,
    member_public,
    SelfWritePreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.self-write.preserve",
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
    const IDENTITY: &'static str = "owned-outputs.self-write.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "self-write-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "self-write-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
