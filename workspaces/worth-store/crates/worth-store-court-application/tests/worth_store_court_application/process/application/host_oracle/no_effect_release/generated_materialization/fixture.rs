//! Minimal generated-output schema and program wiring for Directory Home recovery courts.
#[path = "fixture/operation.rs"]
mod operation;
#[path = "fixture/readiness.rs"]
mod readiness;
#[path = "fixture/records.rs"]
mod records;

pub use operation::OutputProducer;
pub use operation::{
    ArtifactOutput, Generate, GenerateArtifact, GenerateBinding, GenerateHandler, OutputFamily,
    OutputProvider,
};
use readiness::Readiness;
pub use records::{
    Artifact, ArtifactPayload, External, Mapping, Principal, PrincipalValue, Records, Status,
    Target,
};

use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::*;
worth_query_host::facade::worth_query_principal_issuer!(pub GeneratedMaterializationIssuer);
worth_query_host::facade::worth_query_authentication_audience!(pub GeneratedMaterializationAudience);
worth_query_host::facade::worth_query_authentication_method!(pub LocalProcessMethod);

pub struct Catalog;
impl Fragment for Catalog {
    const NAME: &'static str = "Catalog";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Catalog {
    const OWNER: &'static str = "worth.query.early-open-recovery";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for Catalog {
    type Contributions = (Contribution,);
}

pub struct PrincipalBinding;
pub fn principal_binding() -> ApplicationPrincipalBindingRef<
    Catalog,
    PrincipalBinding,
    Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                External::at("External"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at(
                "Status",
            )),
            target: ApplicationPrincipalTargetRequirement::from_relation(Target::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                PrincipalValue::at("Value"),
            ),
        },
    )
}

#[derive(Clone, Debug, WorthValue)]
pub struct SourceRow {
    pub value: u64,
}
worth_query_structured_value_binding!(pub SourceRowBinding for SourceRow { identity: "worth.query.early-open-recovery.source-row.v1" });
worth_query_structured_value_binding!(pub UnitBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(pub SourceQuery for Catalog, identity "worth.query.early-open-recovery.source.v1",
    parameters UnitBinding, result SourceRowBinding, scope Principal => "Principal", name "source");
struct ValueSlot;
worth_query_portable_type!(ValueSlot => "worth.query.early-open-recovery.value.v1");

impl WorthQueryApplicationProjection<Catalog, SourceQuery> for SourceRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, SourceQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            value: row.field(ApplicationQueryResultFieldRef::<
                SourceQuery,
                ValueSlot,
                Catalog,
                PrincipalValue,
            >::new("value", PrincipalValue::at("Value")))?,
        })
    }
}

fn source_query() -> ApplicationQueryDefinitionBuilder<Catalog, SourceQuery, (), SourceRow, Principal>
{
    ApplicationQueryDefinitionBuilder::declare(SourceQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Catalog,
                SourceQuery,
                Principal,
                SourceRow,
                SourceRowBinding,
            >::new(Principal::reference())
            .field(ApplicationQueryResultFieldRef::<
                SourceQuery,
                ValueSlot,
                Catalog,
                PrincipalValue,
            >::new("value", PrincipalValue::at("Value")))
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}

#[derive(Clone, Debug, Default, WorthValue)]
pub struct ReadSource;
worth_query_structured_value_binding!(pub ReadSourceBinding for ReadSource { identity: "worth.query.early-open-recovery.read-source.v1" });

pub struct SourceBinding;
impl ApplicationQueryBinding<Catalog> for SourceBinding {
    type Input = ReadSource;
    type InputBinding = ReadSourceBinding;
    type Query = SourceQuery;
    type ParameterBinding = UnitBinding;
    type ResultBinding = SourceRowBinding;
    type ScopeBinding = ApplicationQueryPrincipalScope<PrincipalValue>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.source-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
    fn scope_field() -> PrincipalValue {
        PrincipalValue::at("Value")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        PrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        principal_binding()
    }
}

impl ApplicationQueryIntent<Catalog> for ReadSource {
    type Binding = SourceBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<SourceQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ApplicationQueryPrincipalScope<PrincipalValue> {
        ApplicationQueryPrincipalScope::new(PrincipalValue::at("Value"))
    }
}

#[derive(Clone)]
pub struct OutputDemand;
impl WorthQueryApplicationOutputDemand<Catalog> for OutputDemand {
    type OutputFamily = OutputFamily;
    fn source_intent(&self) -> ReadSource {
        ReadSource
    }
}

pub struct EntryFeature;
pub struct ParentFeature;
pub struct EntryOutput;
pub struct ParentInput;
pub struct StartConnection;
impl ApplicationFeature<Catalog> for EntryFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.entry.v1";
}
impl ApplicationFeature<Catalog> for ParentFeature {
    type Inputs = ApplicationFeatureInputList<ParentInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.parent.v1";
}
impl ApplicationOutputPort<Catalog, EntryFeature> for EntryOutput {
    type Value = SourceRowBinding;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.entry-output.v1";
}
impl ApplicationInputPort<Catalog, ParentFeature> for ParentInput {
    type Value = SourceRowBinding;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.parent-input.v1";
    const REQUIRED: bool = false;
}
impl ApplicationConnectionIdentity for StartConnection {
    const IDENTITY: &'static str = "worth.query.early-open-recovery.start.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, EntryFeature, ParentFeature>
    for StartConnection
{
}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for StartConnection {
    type Source = GenerateBinding;
    type Demand = OutputDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &Generate,
    ) -> Result<OutputDemand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(OutputDemand)
    }
}
pub type OutputRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        Catalog,
        EntryFeature,
        EntryOutput,
        ParentFeature,
        ParentInput,
        StartConnection,
    >,
    ApplicationOutputLeaf,
>;

pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.early-open-recovery.members.v1");
    type Members = (Records, Contracts);
}
pub struct Contracts;
impl Registration<Catalog> for Contracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Catalog>, ApplicationSchemaDeclarationDenial>
    {
        let op = GenerateArtifact::reference();
        Ok(builder
            .principal_binding(principal_binding())
            .application_query(source_query())
            .application_query_binding::<SourceBinding>()
            .operation(op.definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(op, 64)
            .operation_read_entity(op, Principal::reference())
            .operation_read_field(op, PrincipalValue::at("Value"))
            .operation_create(op, Artifact::reference())
            .operation_write(op, ArtifactPayload::at("Payload"))
            .application_mutation_binding::<GenerateBinding>())
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<OutputRoot>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.early-open-recovery.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Catalog, EntryFeature>()
                .provides::<EntryOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<Catalog, ParentFeature>()
                .conditional_operation::<GenerateArtifact>()
                .finish(),
        ]
    }
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<OutputProducer>()?;
        contracts.conditional::<Readiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<GenerateBinding, _>(GenerateHandler)?;
        setup.producer::<OutputProducer>(OutputProvider)?;
        setup.conditional::<Readiness>(())
    }
}
pub fn seed_principal(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bootstrap.bind_principal(
        &installed.principal_binding(principal_binding()).unwrap(),
        WorthQueryApplicationPrincipalKey::new("early-recovery-reader").unwrap(),
        1u64,
        WorthQueryExternalPrincipalIdentity::new(GeneratedMaterializationIssuer, "reader").unwrap(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    Ok(())
}
