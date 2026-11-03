//! Complete existing schema and typed declarations for pure package composition.
use crate::domain_computation::primary_graph::application_contribution::contracts::tests::package_composition::*;
use worth_foundational::facade::AspectMask;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::ProjectionMask;
use worth_query_declaration::facade::authoring::AspectFieldSelector;
use worth_query_declaration::facade::authoring::AuthoredQueryBundleRequest;
use worth_query_declaration::facade::authoring::AuthoredResultShapeField;
use worth_query_declaration::facade::authoring::DetailQueryBuilder;
use worth_query_declaration::facade::authoring::DetailResultShapeBuilder;
use worth_query_declaration::facade::authoring::RootEntityKey;
use worth_query_declaration::facade::binding::QueryBindingDescriptor;
use worth_query_declaration::facade::canonicalization::canonicalize_request;

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
    // Derive the foundational contract and binding from this actual schema's
    // admitted native catalog instead of inventing a projection contract.
    let admitted =
        domain::WorthQueryInstallationAdmissionProfile::new("composition", "configuration")
            .admit(package().validate().unwrap())
            .unwrap();
    let index = domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(Schema::declaration().unwrap())
    .unwrap();
    let field =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus::reference();
    let native = index
        .native_contracts()
        .aspect(field.entity(), field.aspect())
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
    worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractBuilder::new(
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
        TouchAccountOperation::reference(),
        first.reference(),
    );
    let second_binding = domain::WorthQueryApplicationConditionalOperationBinding::declare(
        PatchAccountDraftOperation::reference(),
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
        // These are owner-private declaration inputs, not executable producer
        // grants or installed receipts. The package validator is the oracle;
        // runtime installation is covered separately by the existing owner11.
        catalog.conditionals.insert(
            format!("conditional-{ordinal}"),
            DeclaredConditionalBinding {
                owner: Schema::OWNER.to_owned(),
                contract,
                required_producers: vec![format!("producer-{ordinal}")],
                binding_type: match ordinal {
                    0 => TypeId::of::<TouchAccountOperation>(),
                    1 => TypeId::of::<PatchAccountDraftOperation>(),
                    _ => TypeId::of::<TouchAccountInput>(),
                },
                operation_type: match ordinal {
                    1 => TypeId::of::<PatchAccountDraftOperation>(),
                    _ => TypeId::of::<TouchAccountOperation>(),
                },
                installed_type: match ordinal {
                    0 => TypeId::of::<TouchAccountInput>(),
                    _ => TypeId::of::<PatchAccountDraftInput>(),
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
