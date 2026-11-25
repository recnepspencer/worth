//! Schema, 2-coupled-member action, and program wiring for original-write flush interruption.
#[path = "fixture/members.rs"]
pub mod members;
#[path = "fixture/records.rs"]
pub mod records;

pub use members::{first, second};
pub use records::{
    ArtifactA, ArtifactB, External, Mapping, Model, ModelData, ModelKey, PayloadA, PayloadB,
    Principal, PrincipalValue, Records, Status, Target,
};

use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::{
    worth_query_application_query, worth_query_mutation_binding, worth_query_operation,
    worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::*;
worth_query_host::facade::worth_query_principal_issuer!(pub CoupledFlushIssuer);
worth_query_host::facade::worth_query_authentication_audience!(pub CoupledFlushAudience);
worth_query_host::facade::worth_query_authentication_method!(pub LocalProcessMethod);

pub struct Catalog;
impl Fragment for Catalog {
    const NAME: &'static str = "CoupledCatalog";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Catalog {
    const OWNER: &'static str = "worth.query.coupled-flush";
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
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Catalog,
        Self,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        principal_binding()
    }
}
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
    pub data: i64,
}
worth_query_structured_value_binding!(pub SourceRowBinding for SourceRow { identity: "worth.query.coupled-flush.source-row.v1" });
worth_query_structured_value_binding!(pub UnitBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(pub SourceQuery for Catalog, identity "worth.query.coupled-flush.source.v1",
    parameters UnitBinding, result SourceRowBinding, scope Model => "Model", name "source");
struct DataSlot;
worth_query_portable_type!(DataSlot => "worth.query.coupled-flush.data.v1");

impl WorthQueryApplicationProjection<Catalog, SourceQuery> for SourceRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, SourceQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            data: row.field(ApplicationQueryResultFieldRef::<
                SourceQuery,
                DataSlot,
                Catalog,
                ModelData,
            >::new("data", ModelData::at("Data")))?,
        })
    }
}

fn source_query() -> ApplicationQueryDefinitionBuilder<Catalog, SourceQuery, (), SourceRow, Model> {
    ApplicationQueryDefinitionBuilder::declare(SourceQuery::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Catalog,
                SourceQuery,
                Model,
                SourceRow,
                SourceRowBinding,
            >::new(Model::reference())
            .field(ApplicationQueryResultFieldRef::<
                SourceQuery,
                DataSlot,
                Catalog,
                ModelData,
            >::new("data", ModelData::at("Data")))
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
worth_query_structured_value_binding!(pub ReadSourceBinding for ReadSource { identity: "worth.query.coupled-flush.read-source.v1" });

pub struct SourceBinding;
impl ApplicationQueryBinding<Catalog> for SourceBinding {
    type Input = ReadSource;
    type InputBinding = ReadSourceBinding;
    type Query = SourceQuery;
    type ParameterBinding = UnitBinding;
    type ResultBinding = SourceRowBinding;
    type ScopeBinding = ApplicationQueryFieldScope<ModelKey>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.coupled-flush.source-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
    fn scope_field() -> ModelKey {
        ModelKey::at("Key")
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
    fn into_scope(self) -> ApplicationQueryFieldScope<ModelKey> {
        ApplicationQueryFieldScope::new(ModelKey::at("Key"), 1u64)
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct Edit {
    pub data: i64,
}
#[derive(Clone, Debug)]
pub struct Edited;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub enum EditDenial {
    Rejected,
}
worth_query_structured_value_binding!(pub EditInputBinding for Edit { identity: "worth.query.coupled-flush.edit-in.v1" });
worth_query_structured_value_binding!(pub EditResultBinding for Edited { identity: "worth.query.coupled-flush.edit-res.v1" });
worth_query_structured_value_binding!(pub EditDenialBinding for EditDenial { identity: "worth.query.coupled-flush.edit-den.v1" });
worth_query_operation!(pub EditOp for Catalog, input EditInputBinding; access {
    read_entities: [Model];
    read_fields: [ModelKey::at("Key"), ModelData::at("Data")];
    writes: [ModelData::at("Data")];
});
fn edit_scope(_: &Edit) -> u64 {
    1
}
worth_query_mutation_binding!(pub EditBinding for Edit, schema Catalog,
    identity "worth.query.coupled-flush.edit.v1", input EditInputBinding, operation EditOp, result EditResultBinding,
    idempotency String, identity "worth.query.coupled-flush.edit-idem.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Model>, denial EditDenialBinding,
    handler identity "worth.query.coupled-flush.edit-handler.v1", program required,
    coupled_outputs |visitor| { visitor.required::<StartFirst>(); visitor.required::<StartSecond>(); },
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("Key"), value edit_scope,
);
pub struct EditHandler;
impl OperationHandler<Catalog, EditBinding> for EditHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        _: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, EditBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Model>, EditDenial> {
        match (|| {
            let root = reader.resolve_entity(ModelKey::at("Key"), 1u64)?;
            reader.field(&root, ModelData::at("Data"))?;
            reader.mutation_target(&root)
        })() {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        target: WorthQueryInvariantMutationTarget<Catalog, Model>,
        writer: &mut CandidateWriter<'_, Catalog, EditBinding>,
    ) -> HandlerResult<Edited, EditDenial> {
        match writer
            .projected_entity(&target)
            .and_then(|entity| writer.write_field(&entity, ModelData::at("Data"), input.data))
        {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(c) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(c)),
        }
    }
}

pub struct EntryFeature;
pub struct DerivedFeature;
pub struct EntryOutput;
pub struct FirstInput;
pub struct SecondInput;
pub struct StartFirst;
pub struct StartSecond;
impl ApplicationFeature<Catalog> for EntryFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.coupled-flush.entry.v1";
}
impl ApplicationFeature<Catalog> for DerivedFeature {
    type Inputs = ApplicationFeatureInputList<
        FirstInput,
        ApplicationFeatureInputList<SecondInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "worth.query.coupled-flush.derived.v1";
}
impl ApplicationOutputPort<Catalog, EntryFeature> for EntryOutput {
    type Value = SourceRowBinding;
    const IDENTITY: &'static str = "worth.query.coupled-flush.entry-out.v1";
}
impl ApplicationInputPort<Catalog, DerivedFeature> for FirstInput {
    type Value = SourceRowBinding;
    const IDENTITY: &'static str = "worth.query.coupled-flush.first-in.v1";
    const REQUIRED: bool = false;
}
impl ApplicationInputPort<Catalog, DerivedFeature> for SecondInput {
    type Value = SourceRowBinding;
    const IDENTITY: &'static str = "worth.query.coupled-flush.second-in.v1";
    const REQUIRED: bool = false;
}
impl ApplicationConnectionIdentity for StartFirst {
    const IDENTITY: &'static str = "worth.query.coupled-flush.start-first.v1";
}
impl ApplicationConnectionIdentity for StartSecond {
    const IDENTITY: &'static str = "worth.query.coupled-flush.start-second.v1";
}
impl ApplicationOccurrenceConnectionBinding<Catalog, EntryFeature, DerivedFeature> for StartFirst {}
impl ApplicationOccurrenceConnectionBinding<Catalog, EntryFeature, DerivedFeature> for StartSecond {}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for StartFirst {
    type Source = first::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(_: &first::GenInput) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> { Ok(first::Demand) }
}
impl WorthQueryApplicationRequiredOutputConnection<Catalog> for StartSecond {
    type Source = second::Binding;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(_: &second::GenInput) -> Result<second::Demand, WorthQueryRequiredOutputConnectionDenial> { Ok(second::Demand) }
}
impl WorthQueryApplicationRequiredOutputSource<Catalog, StartFirst> for EditBinding {
    fn demand_from_source(_: &Edit) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> { Ok(first::Demand) }
}
impl WorthQueryApplicationRequiredOutputSource<Catalog, StartSecond> for EditBinding {
    fn demand_from_source(_: &Edit) -> Result<second::Demand, WorthQueryRequiredOutputConnectionDenial> { Ok(second::Demand) }
}
pub type FirstTree = ApplicationDemandOutputGraph<ApplicationConnectionRef<Catalog, EntryFeature, EntryOutput, DerivedFeature, FirstInput, StartFirst>, ApplicationOutputLeaf>;
pub type SecondTree = ApplicationDemandOutputGraph<ApplicationConnectionRef<Catalog, EntryFeature, EntryOutput, DerivedFeature, SecondInput, StartSecond>, ApplicationOutputLeaf>;
pub struct Completion;
impl<B: ApplicationMutationBinding<Catalog>> ApplicationActionDelivery<Catalog, B> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.coupled-flush.completion.v1";
}

pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.coupled-flush.members.v1");
    type Members = (Records, Contracts);
}
pub struct Contracts;
impl Registration<Catalog> for Contracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Catalog>, ApplicationSchemaDeclarationDenial> {
        let builder = builder
            .principal_binding(principal_binding())
            .application_query(source_query())
            .application_query_binding::<SourceBinding>()
            .declared_operation::<EditOp>(EditOp::reference().definition().no_external_effect().no_aftermath().finish())
            .application_mutation_binding::<EditBinding>();
        let builder = first::register(builder);
        Ok(second::register(builder))
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(FirstTree, SecondTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.coupled-flush.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Catalog, EntryFeature>()
                .provides::<EntryOutput>().action::<EditBinding, Completion>().finish_action().finish(),
            ApplicationFeatureSpec::root::<Catalog, DerivedFeature>()
                .conditional_operation::<first::Operation>()
                .conditional_operation::<second::Operation>()
                .finish(),
        ]
    }
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<first::Producer>()?;
        contracts.producer::<second::Producer>()?;
        contracts.conditional::<first::Readiness>()?;
        contracts.conditional::<second::Readiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<EditBinding, _>(EditHandler)?;
        setup.handler::<first::Binding, _>(first::Handler)?;
        setup.handler::<second::Binding, _>(second::Handler)?;
        setup.producer::<first::Producer>(first::Provider)?;
        setup.producer::<second::Producer>(second::Provider)?;
        setup.conditional::<first::Readiness>(())?;
        setup.conditional::<second::Readiness>(())
    }
}
pub fn seed_principal(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    bootstrap.bind_principal(
        &installed.principal_binding(principal_binding()).unwrap(),
        WorthQueryApplicationPrincipalKey::new("coupled-flush-reader").unwrap(),
        1u64,
        WorthQueryExternalPrincipalIdentity::new(CoupledFlushIssuer, "reader").unwrap(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    bootstrap.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("model-1").unwrap(),
        )
        .field(ModelKey::at("Key"), 1u64)
        .field(ModelData::at("Data"), 1i64),
    )
}
