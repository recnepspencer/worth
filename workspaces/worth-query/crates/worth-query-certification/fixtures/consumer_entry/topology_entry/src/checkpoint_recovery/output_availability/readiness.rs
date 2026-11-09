use super::members::QuantitySnapshot;
use super::*;
use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{binding::Input, binding::PublishQuantity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanarReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub PlanarReadyNode in PlanarReadinessDomain, PlanarReadinessOperation,
    PlanarReadinessFamily => operation "availability-output-ready"
);

pub(super) struct Readiness<const INITIAL: bool>;

impl<const INITIAL: bool> Readiness<INITIAL> {
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            PlanarReadinessDomain,
            PlanarReadinessOperation,
            PlanarReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishQuantity,
        Input,
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishQuantity::reference::<Schema>(),
            definition.reference(),
        )
    }
}

impl<const INITIAL: bool> application_contribution::WorthQueryApplicationConditionalBinding<Schema>
    for Readiness<INITIAL>
{
    type Configuration = ();
    type Installed = ();
    type Operation = PublishQuantity;

    const IDENTITY: &'static str = if INITIAL {
        "availability-initial-readiness"
    } else {
        "availability-preserve-readiness"
    };
    const REQUIRED_PRODUCERS: &'static [&'static str] = if INITIAL {
        &[producer::Initial::IDENTITY]
    } else {
        &[producer::Preserve::IDENTITY]
    };

    fn package_contract(
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let contract = operation_contract(declaration)?;
        let binding = Self::conditional_binding(contract.definition());
        Ok(contract.into_package_contract(binding.portable().clone()))
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let contract = operation_contract(installation.installed_schema().installed_declaration())?;
        let binding = Self::conditional_binding(contract.definition());

        let operation = installation
            .installed_schema()
            .installed_operation(PublishQuantity::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(PlanarReadyNode::reference())
            .unwrap();
        if INITIAL {
            installation.bind_output_readiness::<producer::Initial, _, _, _, _, _, _>(node, 0)
        } else {
            installation.bind_output_readiness::<producer::Preserve, _, _, _, _, _, _>(node, 0)
        }
    }
}
pub(super) fn operation_contract(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    application_contribution::WorthQueryOutputReadinessContract<
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    >,
    application_contribution::WorthQueryOutputReadinessContractDenial,
> {
    application_contribution::WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        QuantitySnapshot::at("QuantitySnapshot"),
        domain::WorthQueryDomainOperationIdentity::new("availability-output-readiness", 1),
        "availability-output-ready",
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("availability-readiness").unwrap(),
        "availability-readiness-v1",
    )?
    .field_dependency(
        declaration,
        QuantitySnapshot::at("QuantitySnapshot"),
        application_contribution::WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed()
}

pub(super) fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(
        RootEntityKey::new(QuantitySnapshot::at("QuantitySnapshot").entity()).unwrap(),
    )
    .project(
        AspectFieldSelector::new(
            QuantitySnapshot::at("QuantitySnapshot").aspect(),
            QuantitySnapshot::at("QuantitySnapshot").field(),
        )
        .unwrap(),
    )
    .build()
    .unwrap()
    .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                QuantitySnapshot::at("QuantitySnapshot").aspect(),
                QuantitySnapshot::at("QuantitySnapshot").field(),
                "snapshot",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
