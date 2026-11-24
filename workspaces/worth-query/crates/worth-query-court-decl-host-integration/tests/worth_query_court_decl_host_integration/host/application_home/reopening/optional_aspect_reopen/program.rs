//! Readiness, contracts, program definition, and Directory Home opener for optional-aspect reopen.
use super::{ops::*, schema::*};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
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
                    .project(AspectFieldSelector::new(DerivedTag::at("DerivedTag").aspect(), "DerivedTag").unwrap())
                    .build().unwrap().into_raw();
                let s = DetailResultShapeBuilder::new()
                    .field(AuthoredResultShapeField::new(DerivedTag::at("DerivedTag").aspect(), "DerivedTag", "tag").unwrap())
                    .build().unwrap().into_raw();
                canonicalize_request(AuthoredQueryBundleRequest::for_ordinary_read(q, s, QueryBindingDescriptor::new()).unwrap()).unwrap()
            }
            fn contract(decl: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryOutputReadinessContract<Domain, Op, Fam>, WorthQueryOutputReadinessContractDenial> {
                WorthQueryOutputReadinessContractBuilder::for_field(
                    decl, DerivedTag::at("DerivedTag"), domain::WorthQueryDomainOperationIdentity::new($node_name, 1),
                    $node_name, query_bundle(), domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
                    domain::WorthQueryExecutionStrategyName::new($node_name).unwrap(), $id,
                )?
                .field_dependency(decl, DerivedTag::at("DerivedTag"), WorthQueryReadinessFieldUse::DeliveryAndEligibility)?
                .build_typed()
            }
            fn cond_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<OptionalAspectSchema, $op, $input, Domain, Op, Fam> {
                let def = contract(OptionalAspectSchema::declaration().unwrap().erased()).unwrap().definition().clone();
                domain::WorthQueryApplicationConditionalOperationBinding::declare($op::reference(), def.reference())
            }
            impl WorthQueryApplicationConditionalBinding<OptionalAspectSchema> for Readiness {
                type Configuration = ();
                type Installed = ();
                type Operation = $op;
                const IDENTITY: &'static str = $id;
                const REQUIRED_PRODUCERS: &'static [&'static str] = &[<$producer as WorthQueryApplicationProducerBinding<OptionalAspectSchema>>::IDENTITY];
                fn package_contract(decl: &ErasedApplicationSchemaDeclaration) -> Result<WorthQueryApplicationConditionalPackageContract, WorthQueryOutputReadinessContractDenial> {
                    Ok(contract(decl)?.into_package_contract(cond_binding().portable().clone()))
                }
                fn install(
                    _: (), _: &WorthQueryApplicationConditionalProducerAccess<'_, OptionalAspectSchema>,
                    installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<OptionalAspectSchema>,
                ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                    let op = installation.installed_schema().installed_operation($op::reference()).unwrap();
                    let node = installation.installed_packages().bind_conditional_application_operation(op, &cond_binding()).unwrap().bind_node(Node::reference()).unwrap();
                    let c = contract(installation.installed_schema().installed_declaration()).unwrap();
                    let ord = c.delivery_ordinal(installation.installed_schema(), node.declaration(), DerivedTag::at("DerivedTag")).unwrap();
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
    "optional-aspect-reopen.init-ready.v1",
    "opt-init-ready"
);
readiness_binding!(
    preserve_ready,
    PreserveProduce,
    PreserveInput,
    PreserveProducer,
    "optional-aspect-reopen.keep-ready.v1",
    "opt-keep-ready"
);

pub struct Contracts;
impl Registration<OptionalAspectSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<OptionalAspectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OptionalAspectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(OperatorPrincipalBinding::reference())
            .application_query_binding::<AnchorReadBinding>()
            .application_query(anchor_query_definition())
            .operation(
                InitialProduce::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(InitialProduce::reference(), 16)
            .operation_read_field(InitialProduce::reference(), AnchorKey::at("AnchorKey"))
            .operation_read_field(
                InitialProduce::reference(),
                SourceSelection::at("SourceSelection"),
            )
            .operation_create(InitialProduce::reference(), Derived::reference())
            .operation_write(InitialProduce::reference(), DerivedKey::at("DerivedKey"))
            .operation_write(InitialProduce::reference(), DerivedTag::at("DerivedTag"))
            .application_mutation_binding::<InitialBinding>()
            .operation(
                PreserveProduce::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(PreserveProduce::reference(), 16)
            .operation_read_field(PreserveProduce::reference(), AnchorKey::at("AnchorKey"))
            .operation_read_field(
                PreserveProduce::reference(),
                SourceSelection::at("SourceSelection"),
            )
            .operation_read_entity(PreserveProduce::reference(), Derived::reference())
            .operation_read_field(PreserveProduce::reference(), DerivedTag::at("DerivedTag"))
            .operation_write(PreserveProduce::reference(), DerivedTag::at("DerivedTag"))
            .application_mutation_binding::<PreserveBinding>()
            .operation(
                MaterializeOther::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(MaterializeOther::reference(), 16)
            .operation_read_field(MaterializeOther::reference(), AnchorKey::at("AnchorKey"))
            .operation_read_field(
                MaterializeOther::reference(),
                SourceContent::at("SourceContent"),
            )
            .operation_write(
                MaterializeOther::reference(),
                SourceContent::at("SourceContent"),
            )
            .application_mutation_binding::<MaterializeBinding>())
    }
}

impl WorthQueryApplicationContribution<OptionalAspectSchema> for OptionalAspectContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<OptionalAspectSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<InitialProducer>()?;
        contracts.producer::<PreserveProducer>()?;
        contracts.conditional::<initial_ready::Readiness>()?;
        contracts.conditional::<preserve_ready::Readiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, OptionalAspectSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<InitialBinding, _>(Handler)?;
        setup.handler::<PreserveBinding, _>(Handler)?;
        setup.handler::<MaterializeBinding, _>(Handler)?;
        setup.producer::<InitialProducer>(Provider)?;
        setup.producer::<PreserveProducer>(Provider)?;
        setup.conditional::<initial_ready::Readiness>(())?;
        setup.conditional::<preserve_ready::Readiness>(())
    }
}

pub struct EntryFeature;
impl ApplicationFeature<OptionalAspectSchema> for EntryFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "optional-aspect-reopen.entry.v1";
}
pub struct OutputPort;
impl ApplicationOutputPort<OptionalAspectSchema, EntryFeature> for OutputPort {
    type Value = KeyRowBinding;
    const IDENTITY: &'static str = "optional-aspect-reopen.output-port.v1";
}
pub struct RootFeature;
impl ApplicationFeature<OptionalAspectSchema> for RootFeature {
    type Inputs = ApplicationFeatureInputList<RootInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "optional-aspect-reopen.root.v1";
}
pub struct RootInput;
impl ApplicationInputPort<OptionalAspectSchema, RootFeature> for RootInput {
    type Value = KeyRowBinding;
    const IDENTITY: &'static str = "optional-aspect-reopen.root-input.v1";
    const REQUIRED: bool = true;
}
pub struct StartConn;
impl ApplicationConnectionIdentity for StartConn {
    const IDENTITY: &'static str = "optional-aspect-reopen.start.v1";
}
impl ApplicationOccurrenceConnectionBinding<OptionalAspectSchema, EntryFeature, RootFeature>
    for StartConn
{
}
impl WorthQueryApplicationRequiredOutputConnection<OptionalAspectSchema> for StartConn {
    type Source = MaterializeBinding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &MaterializeInput,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<OptionalAspectSchema, StartConn>
    for MaterializeBinding
{
    fn demand_from_source(
        input: &MaterializeInput,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        <StartConn as WorthQueryApplicationRequiredOutputConnection<OptionalAspectSchema>>::demand_from_source(input)
    }
}
pub type Root = ApplicationDemandOutputGraph<
    ApplicationConnectionRef<
        OptionalAspectSchema,
        EntryFeature,
        OutputPort,
        RootFeature,
        RootInput,
        StartConn,
    >,
    ApplicationOutputLeaf,
>;
pub struct Program;
impl ApplicationProgramDefinition<OptionalAspectSchema> for Program {
    type Contributions = (OptionalAspectContribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("optional-aspect-reopen.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<OptionalAspectSchema, EntryFeature>()
                .provides::<OutputPort>()
                .native_action::<MaterializeBinding>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<OptionalAspectSchema, RootFeature>()
                .conditional_operation::<InitialProduce>()
                .conditional_operation::<PreserveProduce>()
                .finish(),
        ]
    }
}

pub fn open_directory(
    dir: std::path::PathBuf,
    initialized: Arc<AtomicUsize>,
) -> DirectoryHomeGuard<OptionalAspectSchema, Program> {
    let schema = OptionalAspectSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<OptionalAspectSchema, Program>::begin()
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
            move |_, graph, installed| {
                initialized.fetch_add(1, Ordering::SeqCst);
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
                // Seed Anchor WITHOUT OptionalSource so the optional aspect starts absent.
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Anchor::reference(),
                        WorthQueryApplicationEntityKey::new("anchor-a").unwrap(),
                    )
                    .field(AnchorKey::at("AnchorKey"), "anchor-a".to_owned()),
                )?;
                Ok(())
            },
        )
        .open_in_advancement(phase)
    })
    .expect("optional-aspect Directory Home opens");
    DirectoryHomeGuard::wrap(home, dir)
}
