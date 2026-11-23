//! A decision-input that also reads, through `FirstOf`, every output its
//! source model owns and requires each one's value (PR C C4, #277 court).
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
const PROFILE: &str = "self-relation";
thread_local! {
    /// Only traverse to the model's outputs, never read their values (per
    /// court, on the court's own thread).
    pub static TRAVERSE_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Read model `a`'s outputs' models through the derived `FirstSiblings`
    /// instead of `FirstOf` (#277 court 8): Native's derived traversal reads the
    /// current image and is not overlay-aware (per court).
    pub static DERIVED_READ: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Initial finds its prior output through its output role, never through a
    /// key selection, so its create's unique `FirstKey` is observed only by the
    /// intrinsic unique-write dependency (per court; #277 r4 A).
    pub static BLIND_PRIOR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// With `DERIVED_READ`, ignore the derived read's denial (per court).
    pub static SWALLOW_DERIVED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// A model's outputs' models: `FirstOf` reversed, then `FirstOf`. Its
/// traversal from model `a` probes `a`'s incoming `FirstOf` list.
pub struct FirstSiblings;
impl FirstSiblings {
    pub fn reference() -> ApplicationDerivedRelationRef<Schema, Self, SourceEntity, SourceEntity> {
        ApplicationDerivedRelationRef::from_schema_identifiers("FirstSiblings", "Model", "Model")
    }
}
impl OperationReads<initial::Operation> for FirstSiblings {}
impl OperationReads<preserve::Operation> for FirstSiblings {}
pub fn output_key(row: &query::Row) -> String {
    format!("first:{}", row.key)
}
pub fn derived(value: u64) -> u64 {
    value
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
    SelfRelationInitial,
    false,
    Initial,
    WorthQueryCreateOutput,
    create_output,
    "owned-outputs.self-relation.initial",
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
    SelfRelationPreserve,
    true,
    Preserve,
    WorthQueryPreserveOutput,
    preserve_output,
    "owned-outputs.self-relation.preserve",
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use worth_query_decl::facade::application_schema::{
        ApplicationDerivedRelation, ApplicationRelationGraphLaws,
    };
    preserve::declare(initial::declare(schema))
        .derived_relation(
            FirstSiblings::reference(),
            ApplicationDerivedRelation::relation(FirstOf::reference())
                .reversed()
                .then(FirstOf::reference()),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .operation_read_derived_relation(
            initial::Operation::reference(),
            FirstSiblings::reference(),
        )
        .operation_read_derived_relation(
            preserve::Operation::reference(),
            FirstSiblings::reference(),
        )
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
    const IDENTITY: &'static str = "owned-outputs.self-relation.feature";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "self-relation-source";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "self-relation-result";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .provides::<OutputPort>()
        .conditional_operation::<initial::Operation>()
        .conditional_operation::<preserve::Operation>()
        .finish()
}
