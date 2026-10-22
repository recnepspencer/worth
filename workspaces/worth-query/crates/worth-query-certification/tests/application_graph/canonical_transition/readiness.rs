//! Native readiness contract for the installed canonical-derived producer.

use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{application_contribution, domain};

use super::{
    producer::{Input, Produce, Producer},
    schema::CanonicalSchema,
};

#[path = "readiness/preserve.rs"]
pub mod preserve;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedReadinessFamily;

worth_query_host::facade::worth_query_conditional_node!(
    pub DerivedReadyNode in DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily => operation "canonical-derived-ready"
);

pub struct DerivedReadiness;

impl DerivedReadiness {
    fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
        CanonicalSchema,
        Produce,
        Input,
        DerivedReadinessDomain,
        DerivedReadinessOperation,
        DerivedReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            Produce::reference(),
            operation_definition().reference(),
        )
    }
}

impl application_contribution::WorthQueryApplicationConditionalBinding<CanonicalSchema>
    for DerivedReadiness
{
    type Configuration = ();
    type Installed = ();
    type Operation = Produce;

    const IDENTITY: &'static str = "canonical-transition.readiness.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[
        <Producer as application_contribution::WorthQueryApplicationProducerBinding<
            CanonicalSchema,
        >>::IDENTITY,
    ];

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
                DerivedReadyNode::reference().node_identity(),
            )
        })
    }

    fn install(
        _: (),
        _: &application_contribution::WorthQueryApplicationConditionalProducerAccess<
            '_,
            CanonicalSchema,
        >,
        installation: &mut worth_query_host::facade::primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<CanonicalSchema>,
    ) -> Result<
        (),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let operation = installation
            .installed_schema()
            .installed_operation(Produce::reference())
            .expect("the canonical derived publication operation is installed");
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &Self::conditional_binding())
            .expect("the canonical derived conditional binding is installed")
            .bind_node(DerivedReadyNode::reference())
            .expect("the published-output readiness node is installed");
        installation.bind_output_readiness::<Producer, _, _, _, _, _, _>(node, 0)
    }
}

fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    DerivedReadinessDomain,
    DerivedReadinessOperation,
    DerivedReadinessFamily,
> {
    let dependency = output_dependency();
    application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("canonical-derived-readiness", 1),
        "canonical-derived-ready",
        output_projection(),
        canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("published-output").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("canonical-derived-readiness").unwrap(),
        "canonical-derived-readiness-v1",
    )
    .semantic_reads([output_projection()])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .expect("the canonical derived readiness declaration is canonical")
}

fn output_dependency() -> domain::WorthQuerySemanticTruthDependency {
    // The native producer resolves readiness's SourceRecord from its committed
    // output role. Here that is the newly created Derived record, not Document.
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        derived_contract(),
        output_mask(),
        domain::AspectBinding::EntityField {
            // Installation maps the complete aspect into this native carrier;
            // output_mask still selects the logical produced canonical value field.
            field: domain::FieldKey::new("DerivedFacts").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn derived_contract() -> domain::AspectContract {
    use worth_foundational::facade::aspects;
    use worth_query_host::facade::declaration::application_schema::{
        ApplicationAspectValueMeaning, ApplicationFieldPresence, ApplicationFieldValueMeaning,
        ApplicationSchemaMember,
    };
    let declaration = CanonicalSchema::declaration().unwrap();
    let aspect = ApplicationAspectValueMeaning::from_members(declaration.erased().members())
        .unwrap()
        .into_iter()
        .find(|aspect| aspect.entity() == "Derived" && aspect.aspect() == "DerivedFacts")
        .expect("the derived aspect is declared");
    let family = |field: &ApplicationFieldValueMeaning| match field.member() {
        ApplicationSchemaMember::Field { scalar_family, .. } => *scalar_family,
        _ => unreachable!("meaning is a declared field"),
    };
    let mut fields = aspect.fields().iter();
    let first = fields.next().expect("the derived aspect has fields");
    let mut shape = match first.presence() {
        ApplicationFieldPresence::Required => aspects()
            .struct_fields()
            .required(first.name(), family(first)),
        ApplicationFieldPresence::Optional => aspects()
            .struct_fields()
            .optional(first.name(), family(first)),
    };
    for field in fields {
        shape = match field.presence() {
            ApplicationFieldPresence::Required => shape.required(field.name(), family(field)),
            ApplicationFieldPresence::Optional => shape.optional(field.name(), family(field)),
        };
    }
    let shape = shape.finish().unwrap();
    let shape = domain::StructAspectShape::new(shape.fields().iter().cloned().map(|field| {
        let meaning = aspect
            .fields()
            .iter()
            .find(|meaning| meaning.name() == field.key().as_str())
            .expect("shape contains exactly the declared derived fields");
        match meaning.codec_digest() {
            Some(codec) => field.with_codec(codec),
            None => field,
        }
    }))
    .unwrap();
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new(aspect.aspect()).unwrap(),
        aspect.identity(),
        aspect.revision(),
        shape,
    )
}
fn output_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("DerivedValueField").unwrap(),
    )])
}

fn output_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(derived_contract(), output_mask())
        .unwrap()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Derived").unwrap())
        .project(AspectFieldSelector::new("DerivedFacts", "DerivedValueField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                "DerivedFacts",
                "DerivedValueField",
                "published_derived_value",
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
