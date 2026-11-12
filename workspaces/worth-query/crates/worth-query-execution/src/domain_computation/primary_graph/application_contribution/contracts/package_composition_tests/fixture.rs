//! Complete typed schema and declaration inputs for pure owner composition tests.
use super::*;
use worth_foundational::facade::{AspectMask, CanonicalFieldPath, FieldKey, ProjectionMask};
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_declaration::facade::authoring::{
    AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField, DetailQueryBuilder,
    DetailResultShapeBuilder, RootEntityKey,
};
use worth_query_declaration::facade::{
    binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
};

pub struct Schema;
struct Model;
struct Item;
struct Label;
struct Bindings;
impl fragment::Fragment for Model {
    const NAME: &'static str = "composition";
    type Bindings = Bindings;
    type Members = (Item,);
}
impl fragment::Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (Label,);
    type Relations = ();
}
impl fragment::Aspect for Label {
    const NAME: &'static str = "label";
    type Row = String;
}
impl fragment::BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}
worth_query_declaration::worth_query_structured_value_binding!(pub Input for () {
    identity: "worth.tests.package-composition-input.v1"
});
worth_query_declaration::worth_query_operation!(pub FirstOperation for Schema, input Input);
worth_query_declaration::worth_query_operation!(pub SecondOperation for Schema, input Input);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.tests.package-composition";
    const NAME: &'static str = "Composition";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let first = FirstOperation::reference();
        let second = SecondOperation::reference();
        let field: fragment::FieldSelection<_, _, _, String> =
            fragment::select_field(Model, Item, Label, "value");
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(Model)?
            .operation_read_field(first, field)
            .operation(
                first
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(second, field)
            .operation(
                second
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .build()
    }
}
#[derive(Clone)]
pub struct Domain;
#[derive(Clone)]
pub struct Ready;
#[derive(Clone)]
pub struct Family;
pub type Definition = domain::WorthQueryDomainOperationDefinition<Domain, Ready, Family>;
pub const NODE: &str = "composition-ready";
pub fn package() -> domain::WorthQueryPortableDomainPackage {
    domain::WorthQueryPortableDomainPackage::new(domain::WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ))
    .application_schema(Schema::declaration().unwrap())
}

pub fn definition(name: &str, lowering: &str) -> Definition {
    // The same native compiler projects descriptive contract/binding meaning
    // from the actual declaration, without constructing installation authority.
    let schema = Schema::declaration().unwrap();
    let field: fragment::FieldSelection<_, _, _, String> =
        fragment::select_field(Model, Item, Label, "value");
    let native =
        domain::declared_application_aspect(schema.erased(), field.entity(), field.aspect())
            .unwrap();
    let mask = AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(
        FieldKey::new(field.field()).unwrap(),
    )]);
    let projection = domain::WorthQueryOperationNativeProjectionContract::new(
        native.contract().clone(),
        mask.clone(),
    )
    .unwrap();
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        native.contract().clone(),
        mask,
        native.binding().clone(),
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap();
    let query = DetailQueryBuilder::new(RootEntityKey::new(field.entity()).unwrap())
        .project(AspectFieldSelector::new(field.aspect(), field.field()).unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(field.aspect(), field.field(), "status").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let canonical = canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap();
    crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new(name, 1),
        NODE,
        projection.clone(),
        canonical,
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("composition-readiness").unwrap(),
        lowering,
    )
    .semantic_reads([projection])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .unwrap()
}

pub fn contracts(first: &Definition, second: &Definition) -> [Contract; 2] {
    let first_binding = domain::WorthQueryApplicationConditionalOperationBinding::declare(
        FirstOperation::reference(),
        first.reference(),
    );
    let second_binding = domain::WorthQueryApplicationConditionalOperationBinding::declare(
        SecondOperation::reference(),
        second.reference(),
    );
    [
        Contract::new(
            first.clone().into_portable(),
            first_binding.portable().clone(),
            NODE,
        ),
        Contract::new(
            second.clone().into_portable(),
            second_binding.portable().clone(),
            NODE,
        ),
    ]
}

pub fn catalog(contracts: Vec<Contract>) -> Catalog {
    let mut catalog = Catalog::default();
    for (ordinal, contract) in contracts.into_iter().enumerate() {
        // Owner-private declaration inputs, not executable producers or installed receipts.
        // The real package validator is the oracle for this pure composition boundary.
        catalog.conditionals.insert(
            format!("conditional-{ordinal}"),
            DeclaredConditionalBinding {
                owner: Schema::OWNER.to_owned(),
                contract,
                required_producers: vec![format!("producer-{ordinal}")],
                binding_type: match ordinal {
                    0 => TypeId::of::<FirstOperation>(),
                    1 => TypeId::of::<SecondOperation>(),
                    _ => TypeId::of::<u64>(),
                },
                operation_type: match ordinal {
                    1 => TypeId::of::<SecondOperation>(),
                    _ => TypeId::of::<FirstOperation>(),
                },
                installed_type: match ordinal {
                    0 => TypeId::of::<()>(),
                    _ => TypeId::of::<String>(),
                },
            },
        );
    }
    catalog
}
pub fn snapshot(catalog: &Catalog) -> Vec<(String, String, Contract, Vec<String>, TypeId, TypeId)> {
    catalog
        .conditionals
        .iter()
        .map(|(identity, declaration)| {
            (
                identity.clone(),
                declaration.owner.clone(),
                declaration.contract.clone(),
                declaration.required_producers.clone(),
                declaration.binding_type,
                declaration.installed_type,
            )
        })
        .collect()
}
