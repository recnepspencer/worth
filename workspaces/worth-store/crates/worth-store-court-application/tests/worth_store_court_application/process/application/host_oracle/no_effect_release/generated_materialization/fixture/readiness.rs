//! Minimal output readiness conditional for Directory Home recovery courts.
use super::{ArtifactPayload, Catalog, Generate, GenerateArtifact, OutputProducer};
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_schema::*, authoring::*, binding::QueryBindingDescriptor,
        canonicalization::{canonicalize_request, CanonicalQueryBundle},
    },
    domain,
    primary_graph::*,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub ReadyNode in ReadinessDomain, ReadinessOperation, ReadinessFamily => operation "early-open-artifact-ready"
);

pub struct Readiness;
type Contract =
    WorthQueryOutputReadinessContract<ReadinessDomain, ReadinessOperation, ReadinessFamily>;
type Binding = domain::WorthQueryApplicationConditionalOperationBinding<
    Catalog,
    GenerateArtifact,
    Generate,
    ReadinessDomain,
    ReadinessOperation,
    ReadinessFamily,
>;

fn binding(contract: &Contract) -> Binding {
    Binding::declare(
        GenerateArtifact::reference(),
        contract.definition().reference(),
    )
}

fn contract(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> Result<Contract, WorthQueryOutputReadinessContractDenial> {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        ArtifactPayload::at("Payload"),
        domain::WorthQueryDomainOperationIdentity::new("early-open-artifact-readiness", 1),
        "early-open-artifact-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("generated-artifact").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("early-open-artifact-readiness").unwrap(),
        "early-open-artifact-readiness-v1",
    )?
    .field_dependency(
        declaration,
        ArtifactPayload::at("Payload"),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

impl WorthQueryApplicationConditionalBinding<Catalog> for Readiness {
    type Configuration = ();
    type Installed = ();
    type Operation = GenerateArtifact;
    const IDENTITY: &'static str = "worth.query.early-open-recovery.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] =
        &[<OutputProducer as WorthQueryApplicationProducerBinding<Catalog>>::IDENTITY];

    fn package_contract(
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        let contract = contract(declaration)?;
        let binding = binding(&contract);
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, Catalog>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Catalog>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let contract = contract(installation.installed_schema().installed_declaration()).unwrap();
        let operation = installation
            .installed_schema()
            .installed_operation(GenerateArtifact::reference())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding(&contract))
            .unwrap()
            .bind_node(ReadyNode::reference())
            .unwrap();
        let ordinal = contract
            .delivery_ordinal(
                installation.installed_schema(),
                node.declaration(),
                ArtifactPayload::at("Payload"),
            )
            .unwrap();
        installation.bind_output_readiness::<OutputProducer, _, _, _, _, _, _>(node, ordinal)
    }
}

fn canonical_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Artifact").unwrap())
        .project(AspectFieldSelector::new("ArtifactFacts", "Payload").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("ArtifactFacts", "Payload", "payload").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
