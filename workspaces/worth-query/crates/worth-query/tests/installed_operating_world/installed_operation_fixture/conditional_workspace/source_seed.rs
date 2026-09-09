use worth_foundational::facade::{
    AspectContract, AspectKey, AspectShape, AspectValue, CanonicalF64, CanonicalFieldPath,
    FieldKey, ScalarAspectType, StructAspectValue,
};
use worth_query::facade::{consumer_kit, domain, runtime};

pub(crate) const SOURCE_KEY: &str = "conditional-source";

pub(crate) fn schema_and_rows(
    dependencies: impl IntoIterator<Item = domain::WorthQuerySemanticTruthDependency>,
) -> (
    consumer_kit::WorthQueryTestBackendSchema,
    Vec<consumer_kit::WorthQueryTestSeedRow>,
) {
    let mut contracts = std::collections::BTreeMap::new();
    let identity = super::super::identity_contract();
    contracts.insert(identity.key().clone(), identity);
    for dependency in dependencies {
        let contract = dependency.contract().clone();
        if let Some(prior) = contracts.insert(contract.key().clone(), contract.clone()) {
            assert_eq!(prior, contract, "fixture source contracts must agree");
        }
    }
    let mut schema = consumer_kit::WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contracts(contracts.values().cloned())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    for contract in contracts
        .values()
        .filter(|contract| contract.key().as_str() != "identity")
    {
        schema = schema
            .aspect(contract.key().as_str(), contract.key().as_str())
            .unwrap();
    }
    let row =
        consumer_kit::WorthQueryTestSeedRow::new(SOURCE_KEY, "Vertex", move |mut mutation| {
            for contract in contracts.into_values() {
                mutation = append_value(mutation, &contract);
            }
            mutation
        })
        .unwrap();
    (schema, vec![row])
}

pub(crate) fn identity_touch() -> runtime::WorthQueryAspectTouch {
    runtime::WorthQueryAspectTouch::aspect_field_path(
        AspectKey::new("identity").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("id").unwrap()),
    )
}

pub(crate) fn seeded_builder(
    builder: consumer_kit::WorthQueryInMemoryTestRuntimeBuilder,
    dependencies: impl IntoIterator<Item = domain::WorthQuerySemanticTruthDependency>,
) -> consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let (schema, rows) = schema_and_rows(dependencies);
    builder
        .with_schema(schema)
        .seed_collection_rows(identity_touch(), rows)
        .unwrap()
}

pub(crate) fn standalone(
    dependency: domain::WorthQuerySemanticTruthDependency,
) -> (
    worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    consumer_kit::WorthQueryTestSeedReceipt,
) {
    let (schema, rows) = schema_and_rows([dependency]);
    schema
        .seeded_relational_source(identity_touch(), rows)
        .unwrap()
}

fn append_value(
    mutation: runtime::WorthQueryAspectMutationBuilder,
    contract: &AspectContract,
) -> runtime::WorthQueryAspectMutationBuilder {
    match contract.shape() {
        AspectShape::Scalar(kind) => mutation.aspect(contract.key().as_str(), scalar(*kind)),
        AspectShape::Struct(shape) => mutation.aspect(
            contract.key().as_str(),
            StructAspectValue::new(
                shape
                    .fields()
                    .iter()
                    .map(|field| (field.key().clone(), scalar(field.value_type()))),
            )
            .unwrap(),
        ),
        other => panic!("conditional fixture requires a native scalar or struct source: {other:?}"),
    }
}

fn scalar(kind: ScalarAspectType) -> AspectValue {
    match kind {
        ScalarAspectType::String => AspectValue::String(SOURCE_KEY.into()),
        ScalarAspectType::Float64 => AspectValue::Float64(CanonicalF64::from_f64(10.0)),
        ScalarAspectType::UInt64 => AspectValue::UInt64(10),
        ScalarAspectType::Int64 => AspectValue::Int64(10),
        ScalarAspectType::Bool => AspectValue::Bool(true),
        other => panic!("conditional fixture must explicitly declare a seed for {other:?}"),
    }
}
