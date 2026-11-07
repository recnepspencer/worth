//! Real completed declarations and installed catalogs, not copied aspect fixtures.
pub use worth_query_declaration::facade::canonicalization::CanonicalQueryBundle;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::native_field_contract::FieldExpectation;
pub use worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractBuilder;
pub use worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryReadinessFieldUse;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::denial;
pub use worth_query_installation::facade as domain;
pub use worth_query_declaration::facade::application_schema as schema;
use schema::fragment::{self};
use schema::{ApplicationScalarValueBinding, ApplicationSchema};
mod composition_refusal;
mod scalar;
use worth_foundational::facade::{AbsenceLaw, AspectShape, WorthValue};
use worth_query_declaration::facade::{
    authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
};

#[derive(Clone, WorthValue)]
struct Payload {
    nested: Vec<Choice>,
}
#[derive(Clone, WorthValue)]
enum Choice {
    Empty,
    Selected { label: String, count: u64 },
}
#[derive(Clone, WorthValue)]
struct ChangedPayload {
    nested: Vec<ChangedChoice>,
}
#[derive(Clone, WorthValue)]
enum ChangedChoice {
    Empty,
    Selected { renamed: String, count: u64 },
}
worth_query_declaration::worth_query_value_binding!(Binding for Payload {
    identity: "readiness.payload.v1", scalar: String, canonical,
});
worth_query_declaration::worth_query_value_binding!(TwinBinding for Payload {
    identity: "readiness.payload.v1", scalar: String, canonical,
});
worth_query_declaration::worth_query_value_binding!(ChangedBinding for ChangedPayload {
    identity: "readiness.payload.v1", scalar: String, canonical,
});
#[path = "records.rs"]
mod records;
use records::*;
type Record = fragment::FieldSelection<Model<0>, Node<0>, Publication<0>, Payload>;
type Key = fragment::FieldSelection<Model<0>, Node<0>, Identity, String>;
mod changed;
mod twin;
struct Actual;
impl schema::ApplicationSchema for Actual {
    const OWNER: &'static str = "Actual";
    const NAME: &'static str = "Actual";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<0>)?.build()
    }
}
struct Changed;
impl schema::ApplicationSchema for Changed {
    const OWNER: &'static str = "Changed";
    const NAME: &'static str = "Changed";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<1>)?.build()
    }
}
struct Missing;
impl schema::ApplicationSchema for Missing {
    const OWNER: &'static str = "Missing";
    const NAME: &'static str = "Missing";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<3>)?.build()
    }
}

pub fn installed() -> domain::WorthQueryInstalledApplicationSchema<Actual> {
    let declaration = Actual::declaration().unwrap();
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new(Actual::OWNER, Actual::MAJOR, Actual::MINOR),
    )
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted =
        domain::WorthQueryInstallationAdmissionProfile::new("typed-readiness", "fixture")
            .admit(package)
            .unwrap();
    domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
pub fn query() -> CanonicalQueryBundle {
    let request = DetailQueryBuilder::new(RootEntityKey::new("Entity").unwrap())
        .project(AspectFieldSelector::new(Key::at("Key").aspect(), "Key").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(Key::at("Key").aspect(), "Key", "key").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(
            request,
            shape,
            QueryBindingDescriptor::new(),
        )
        .unwrap(),
    )
    .unwrap()
}
pub fn builder(
    declaration: &schema::ErasedApplicationSchemaDeclaration,
) -> WorthQueryOutputReadinessContractBuilder {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        Key::at("Key"),
        domain::WorthQueryDomainOperationIdentity::new("readiness", 1),
        "ready",
        query(),
        domain::WorthQueryOperationProjectionRole::new("realization").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("readiness").unwrap(),
        "typed-readiness-v1",
    )
    .unwrap()
}

#[test]
fn typed_field_uses_complete_optional_aspect_and_real_nested_codec() {
    let installed = installed();
    let field = FieldExpectation::declared_for_test(
        installed.installed_declaration(),
        Record::at("Record"),
    )
    .unwrap();
    field.seal_for_test(&installed).unwrap();
    assert_eq!((*field.aspect_for_test()).absence(), AbsenceLaw::Optional);
    let AspectShape::Struct(shape) = (*field.aspect_for_test()).shape() else {
        panic!("struct aspect")
    };
    assert_eq!(shape.fields().len(), 2);
    let record = shape
        .fields()
        .iter()
        .find(|f| f.key().as_str() == "Record")
        .unwrap();
    let sibling = shape
        .fields()
        .iter()
        .find(|f| f.key().as_str() == "Sibling")
        .unwrap();
    assert_eq!(record.absence(), AbsenceLaw::Required);
    assert_eq!(sibling.absence(), AbsenceLaw::Optional);
    assert!(
        record.codec().is_some(),
        "the actual nested canonical grammar participates"
    );
    let changed = Changed::declaration().unwrap();
    let changed_field =
        FieldExpectation::declared_for_test(changed.erased(), crate::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests::changed::Record::at("Record"))
            .unwrap();
    assert_ne!(
        (*field.aspect_for_test()),
        (*changed_field.aspect_for_test()),
        "nested rename changes consumed readiness contract"
    );
    let value = Payload {
        nested: vec![Choice::Selected {
            label: "δ".into(),
            count: 17,
        }],
    };
    let encoded = Binding::encode(&value).unwrap();
    let schema::ApplicationValue::String(text) = encoded else {
        panic!("canonical text")
    };
    assert_eq!(
        text.raw_text().expect("canonical text is textual"),
        r#"{"nested":[{"Selected":{"label":"δ","count":17}}]}"#
    );
}

#[test]
fn actual_missing_changed_and_wrong_concrete_bindings_refuse_at_field_locus() {
    let installed = installed();
    for declaration in [
        Missing::declaration().unwrap().into_erased(),
        Changed::declaration().unwrap().into_erased(),
    ] {
        let denial =
            FieldExpectation::declared_for_test(&declaration, Record::at("Record")).unwrap_err();
        assert_eq!(
            denial.field_locus_for_test(),
            Some(&schema::ApplicationFieldBindingLocus::new(
                "Entity",
                Record::at("Record").aspect(),
                "Record"
            ))
        );
    }
    // Portable bytes agree, but the actual installed Binding TypeId differs.
    let twin = FieldExpectation::declared_for_test(
        installed.installed_declaration(),
        crate::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests::twin::Record::at("Record"),
    )
    .unwrap();
    let denial = twin.seal_for_test(&installed).unwrap_err();
    assert_eq!(denial.field_locus_for_test().unwrap().field(), "Record");
    assert!(denial.detail_for_test().contains("installed value binding"));
}

#[test]
fn canonical_order_and_distinct_delivery_eligibility_are_preserved() {
    let installed = installed();
    let declaration = installed.installed_declaration();
    let both = WorthQueryReadinessFieldUse::DeliveryAndEligibility;
    let first = builder(declaration)
        .field_dependency(declaration, Record::at("Record"), both)
        .unwrap()
        .field_dependency(declaration, Key::at("Key"), both)
        .unwrap()
        .build_typed::<(), (), ()>()
        .unwrap();
    let reversed = builder(declaration)
        .field_dependency(declaration, Key::at("Key"), both)
        .unwrap()
        .field_dependency(declaration, Record::at("Record"), both)
        .unwrap()
        .build_typed::<(), (), ()>()
        .unwrap();
    assert_eq!((*first.fields_for_test()), (*reversed.fields_for_test()));
    assert_eq!(
        (*first.delivery_fields_for_test()),
        (*reversed.delivery_fields_for_test())
    );
    assert_eq!(
        first.definition().clone().into_portable(),
        reversed.definition().clone().into_portable()
    );
    let node = &first.definition().semantics().conditional_nodes[0];
    let ordinal = first
        .delivery_ordinal(&installed, node, Record::at("Record"))
        .unwrap();
    let expected = FieldExpectation::declared_for_test(declaration, Record::at("Record"))
        .unwrap()
        .dependency_for_test()
        .unwrap();
    assert_eq!(node.dependencies()[ordinal], expected);
    let distinct = builder(declaration)
        .field_dependency(
            declaration,
            Record::at("Record"),
            WorthQueryReadinessFieldUse::Delivery,
        )
        .unwrap()
        .field_dependency(
            declaration,
            Key::at("Key"),
            WorthQueryReadinessFieldUse::Eligibility,
        )
        .unwrap()
        .build_typed::<(), (), ()>()
        .unwrap();
    let node = &distinct.definition().semantics().conditional_nodes[0];
    assert_eq!(node.dependencies().len(), 2);
    assert_eq!(node.condition().dependencies().len(), 1);
    assert_ne!(node.dependencies(), node.condition().dependencies());
    let ordinal = distinct
        .delivery_ordinal(&installed, node, Record::at("Record"))
        .unwrap();
    assert_eq!(node.dependencies()[ordinal], expected);
    assert!(distinct
        .delivery_ordinal(&installed, node, Key::at("Key"))
        .is_err());
}
