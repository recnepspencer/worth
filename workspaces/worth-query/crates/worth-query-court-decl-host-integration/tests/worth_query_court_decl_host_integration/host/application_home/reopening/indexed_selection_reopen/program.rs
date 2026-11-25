//! Readiness, contracts, program definition, and Directory Home opener for indexed-selection reopen.
use super::{ops::*, schema::*};
use worth_query_decl::facade::{
    application_program::*,
    application_schema::{fragment::Registration, *},
    authentication::WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{
    application_contribution::*,
    application_installation::*,
    declaration::{
        authoring::{
            AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
            DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
        },
        binding::QueryBindingDescriptor,
        canonicalization::canonicalize_request,
    },
    domain,
    primary_graph::*,
    *,
};
use worth_query_testkit_document_retention::facade::{
    directory_home::{with_directory_bootstrap, DirectoryHomeGuard},
    document_retention_model::operator_identity::operator_identity,
};

macro_rules! readiness_binding {
    ($mod_name:ident, $op:ident, $input:ty, $producer:ty, $id:literal, $node_name:literal) => {
        pub mod $mod_name {
            use super::*;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct Domain;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct Op;
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct Fam;
            worth_query_host::facade::worth_query_conditional_node!(pub Node in Domain, Op, Fam => operation $node_name);
            pub struct Readiness;
            fn query_bundle() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
                let q = DetailQueryBuilder::new(RootEntityKey::new("Derived").unwrap())
                    .project(AspectFieldSelector::new(DerivedMarker::at("DerivedMarker").aspect(), "DerivedMarker").unwrap())
                    .build().unwrap().into_raw();
                let s = DetailResultShapeBuilder::new()
                    .field(AuthoredResultShapeField::new(DerivedMarker::at("DerivedMarker").aspect(), "DerivedMarker", "marker").unwrap())
                    .build().unwrap().into_raw();
                canonicalize_request(AuthoredQueryBundleRequest::for_ordinary_read(q, s, QueryBindingDescriptor::new()).unwrap()).unwrap()
            }
            fn contract(decl: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryOutputReadinessContract<Domain, Op, Fam>, WorthQueryOutputReadinessContractDenial> {
                WorthQueryOutputReadinessContractBuilder::for_field(
                    decl, DerivedMarker::at("DerivedMarker"), domain::WorthQueryDomainOperationIdentity::new($node_name, 1),
                    $node_name, query_bundle(), domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
                    domain::WorthQueryExecutionStrategyName::new($node_name).unwrap(), $id,
                )?
                .field_dependency(decl, DerivedMarker::at("DerivedMarker"), WorthQueryReadinessFieldUse::DeliveryAndEligibility)?
                .build_typed()
            }
            fn cond_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<IndexedSelectionSchema, $op, $input, Domain, Op, Fam> {
                let def = contract(IndexedSelectionSchema::declaration().unwrap().erased()).unwrap().definition().clone();
                domain::WorthQueryApplicationConditionalOperationBinding::declare($op::reference(), def.reference())
            }
            impl WorthQueryApplicationConditionalBinding<IndexedSelectionSchema> for Readiness {
                type Configuration = ();
                type Installed = ();
                type Operation = $op;
                const IDENTITY: &'static str = $id;
                const REQUIRED_PRODUCERS: &'static [&'static str] = &[<$producer as WorthQueryApplicationProducerBinding<IndexedSelectionSchema>>::IDENTITY];
                fn package_contract(decl: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                    Ok(contract(decl)?.into_package_contract(cond_binding().portable().clone()))
                }
                fn install(
                    _: (), _: &WorthQueryApplicationConditionalProducerAccess<'_, IndexedSelectionSchema>,
                    installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<IndexedSelectionSchema>,
                ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                    let op = installation.installed_schema().installed_operation($op::reference()).unwrap();
                    let node = installation.installed_packages().bind_conditional_application_operation(op, &cond_binding()).unwrap().bind_node(Node::reference()).unwrap();
                    let c = contract(installation.installed_schema().installed_declaration()).unwrap();
                    let ord = c.delivery_ordinal(installation.installed_schema(), node.declaration(), DerivedMarker::at("DerivedMarker")).unwrap();
                    installation.bind_output_readiness::<$producer, _, _, _, _, _, _>(node, ord)
                }
            }
        }
    };
}
readiness_binding!(
    initial_ready,
    InitialProduce,
    InitialInput,
    InitialProducer,
    "indexed-selection-reopen.init-ready.v1",
    "idx-init-ready"
);
readiness_binding!(
    preserve_ready,
    PreserveProduce,
    PreserveInput,
    PreserveProducer,
    "indexed-selection-reopen.keep-ready.v1",
    "idx-keep-ready"
);

pub struct Contracts;
impl Registration<IndexedSelectionSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<IndexedSelectionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IndexedSelectionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(OperatorPrincipalBinding::reference())
            .application_query_binding::<AccountReadBinding>()
            .application_query(account_query_definition())
            .operation(
                InitialProduce::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(InitialProduce::reference(), 16)
            .operation_read_field(InitialProduce::reference(), AccountKey::at("AccountKey"))
            .operation_read_field(
                InitialProduce::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_create(InitialProduce::reference(), Account::reference())
            .operation_write(InitialProduce::reference(), AccountKey::at("AccountKey"))
            .operation_write(
                InitialProduce::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_create(InitialProduce::reference(), Derived::reference())
            .operation_write(InitialProduce::reference(), DerivedKey::at("DerivedKey"))
            .operation_write(
                InitialProduce::reference(),
                DerivedMarker::at("DerivedMarker"),
            )
            .application_mutation_binding::<InitialBinding>()
            .operation(
                PreserveProduce::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(PreserveProduce::reference(), 16)
            .operation_read_field(PreserveProduce::reference(), AccountKey::at("AccountKey"))
            .operation_read_field(
                PreserveProduce::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_read_entity(PreserveProduce::reference(), Derived::reference())
            .operation_read_field(
                PreserveProduce::reference(),
                DerivedMarker::at("DerivedMarker"),
            )
            .operation_write(
                PreserveProduce::reference(),
                DerivedMarker::at("DerivedMarker"),
            )
            .application_mutation_binding::<PreserveBinding>()
            .operation(
                UpdateStatus::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(UpdateStatus::reference(), 16)
            .operation_read_field(UpdateStatus::reference(), AccountKey::at("AccountKey"))
            .operation_read_field(
                UpdateStatus::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_write(
                UpdateStatus::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .application_mutation_binding::<UpdateStatusBinding>())
    }
}

impl WorthQueryApplicationContribution<IndexedSelectionSchema> for IndexedSelectionContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<IndexedSelectionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<InitialProducer>()?;
        contracts.producer::<PreserveProducer>()?;
        contracts.conditional::<initial_ready::Readiness>()?;
        contracts.conditional::<preserve_ready::Readiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, IndexedSelectionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<InitialBinding, _>(Handler)?;
        setup.handler::<PreserveBinding, _>(Handler)?;
        setup.handler::<UpdateStatusBinding, _>(Handler)?;
        setup.producer::<InitialProducer>(Provider)?;
        setup.producer::<PreserveProducer>(Provider)?;
        setup.conditional::<initial_ready::Readiness>(())?;
        setup.conditional::<preserve_ready::Readiness>(())
    }
}

pub struct EntryFeature;
impl ApplicationFeature<IndexedSelectionSchema> for EntryFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "indexed-selection-reopen.entry.v1";
}
pub struct OutputPort;
impl ApplicationOutputPort<IndexedSelectionSchema, EntryFeature> for OutputPort {
    type Value = KeyRowBinding;
    const IDENTITY: &'static str = "indexed-selection-reopen.output-port.v1";
}
pub struct RootFeature;
impl ApplicationFeature<IndexedSelectionSchema> for RootFeature {
    type Inputs = ApplicationFeatureInputList<RootInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "indexed-selection-reopen.root.v1";
}
pub struct RootInput;
impl ApplicationInputPort<IndexedSelectionSchema, RootFeature> for RootInput {
    type Value = KeyRowBinding;
    const IDENTITY: &'static str = "indexed-selection-reopen.root-input.v1";
    const REQUIRED: bool = true;
}
pub struct StartConn;
impl ApplicationConnectionIdentity for StartConn {
    const IDENTITY: &'static str = "indexed-selection-reopen.start.v1";
}
impl ApplicationOccurrenceConnectionBinding<IndexedSelectionSchema, EntryFeature, RootFeature>
    for StartConn
{
}
impl WorthQueryApplicationRequiredOutputConnection<IndexedSelectionSchema> for StartConn {
    type Source = UpdateStatusBinding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &UpdateStatusInput,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<IndexedSelectionSchema, StartConn>
    for UpdateStatusBinding
{
    fn demand_from_source(
        input: &UpdateStatusInput,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        <StartConn as WorthQueryApplicationRequiredOutputConnection<IndexedSelectionSchema>>::demand_from_source(input)
    }
}
pub type Root = ApplicationDemandOutputGraph<
    ApplicationConnectionRef<
        IndexedSelectionSchema,
        EntryFeature,
        OutputPort,
        RootFeature,
        RootInput,
        StartConn,
    >,
    ApplicationOutputLeaf,
>;
pub struct Program;
impl ApplicationProgramDefinition<IndexedSelectionSchema> for Program {
    type Contributions = (IndexedSelectionContribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("indexed-selection-reopen.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IndexedSelectionSchema, EntryFeature>()
                .provides::<OutputPort>()
                .native_action::<UpdateStatusBinding>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<IndexedSelectionSchema, RootFeature>()
                .conditional_operation::<InitialProduce>()
                .conditional_operation::<PreserveProduce>()
                .finish(),
        ]
    }
}

pub fn open_directory(
    dir: std::path::PathBuf,
) -> DirectoryHomeGuard<IndexedSelectionSchema, Program> {
    let schema = IndexedSelectionSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<IndexedSelectionSchema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let limits = WorthQueryApplicationLimits::bounded_serial(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
    .install()
    .unwrap();
    let home = with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(dir.clone()),
            program,
            WorthQueryApplicationProgramRoster::new(),
            schema,
            ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            limits,
            |_, graph, installed| {
                let binding = installed
                    .principal_binding(OperatorPrincipalBinding::reference())
                    .unwrap();
                graph.bind_principal(
                    &binding,
                    WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                    1_u64,
                    operator_identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Account::reference(),
                        WorthQueryApplicationEntityKey::new("acct-1").unwrap(),
                    )
                    .field(AccountKey::at("AccountKey"), "acct-1".to_owned())
                    .field(AccountStatus::at("AccountStatus"), "active".to_owned()),
                )?;
                Ok(())
            },
        )
        .open_in_advancement(phase)
    })
    .expect("indexed-selection Directory Home opens");
    DirectoryHomeGuard::wrap(home, dir)
}
