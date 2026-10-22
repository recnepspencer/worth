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
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        Schema,
        PublishQuantity,
        Input,
        PlanarReadinessDomain,
        PlanarReadinessOperation,
        PlanarReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            PublishQuantity::reference::<Schema>(),
            operation_definition().reference(),
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
        _declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        application_contribution::WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        Ok({
            application_contribution::WorthQueryApplicationConditionalPackageContract::new(
                operation_definition().into_portable(),
                Self::conditional_binding().portable().clone(),
                PlanarReadyNode::reference().node_identity(),
            )
        })
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(PublishQuantity::reference::<Schema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
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
pub(super) fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    PlanarReadinessDomain,
    PlanarReadinessOperation,
    PlanarReadinessFamily,
> {
    let dependency = output_change_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("availability-output-readiness", 1),
        "availability-output-ready",
        output_change_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("availability-readiness").unwrap(),
        "availability-readiness-v1",
    )
    .semantic_reads([output_change_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("planar readiness declaration is canonical")
}

pub(super) fn output_change_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        output_contract(),
        output_change_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("QuantityFacts").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn output_contract() -> domain::AspectContract {
    let field = |name, scalar| {
        domain::FieldDeclaration::new(
            domain::FieldKey::new(name).unwrap(),
            scalar,
            domain::FieldRequirement::Required,
            domain::AbsenceLaw::Required,
            domain::AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    };
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new("QuantityFacts").unwrap(),
        domain::AspectIdentity(0x9174_2731),
        domain::AspectContractRevision(1),
        // Correspondence admits the complete installed aspect, while the mask
        // below selects only the output value whose changes drive readiness.
        domain::StructAspectShape::new([
            field("QuantityKey", domain::ScalarAspectType::String),
            field("QuantitySnapshot", domain::ScalarAspectType::String),
        ])
        .unwrap(),
    )
}

fn output_change_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("QuantitySnapshot").unwrap(),
    )])
}

pub(super) fn output_change_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(
        output_contract(),
        output_change_mask(),
    )
    .unwrap()
}

pub(super) fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("Quantity").unwrap())
        .project(AspectFieldSelector::new("QuantityFacts", "QuantitySnapshot").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new("QuantityFacts", "QuantitySnapshot", "snapshot").unwrap(),
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
