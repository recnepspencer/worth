//! A real source projection consumes both diamond parents before publishing a member.
use super::super::schema::{
    FirstKey as SourceKey, FirstOutput as SourceEntity, FirstValue as SourceValue,
};
use super::*;
type Scope = ApplicationMutationFieldScope<SourceKey>;
fn source_key() -> SourceKey {
    SourceKey::at("FirstKey")
}
fn source_value() -> SourceValue {
    SourceValue::at("FirstValue")
}
const PROFILE: &str = "joined";
pub fn output_key(row: &query::Row) -> String {
    format!("joined:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value
}
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = query::joined::Binding;
    type Entity = Member;
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
    fn source_intent(&self) -> query::joined::Read {
        query::joined::Read(self.0.clone())
    }
}
phase!(
    initial,
    SourceScopeCascade,
    source,
    joined,
    JoinedInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.joined.initial",
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    JoinedOf
);
phase!(
    preserve,
    SourceScopeCascade,
    source,
    joined,
    JoinedPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.joined.preserve",
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    JoinedOf
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

pub struct Join;
pub struct LeftInput;
pub struct RightInput;
impl ApplicationFeature<Schema> for Join {
    type Inputs = ApplicationFeatureInputList<
        LeftInput,
        ApplicationFeatureInputList<RightInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "owned-outputs.diamond.join";
}
impl ApplicationInputPort<Schema, Join> for LeftInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "diamond-left-input";
    const REQUIRED: bool = false;
}
impl ApplicationInputPort<Schema, Join> for RightInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "diamond-right-input";
    const REQUIRED: bool = false;
}

pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Join>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
