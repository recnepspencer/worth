//! Real completed declarations and installed catalogs, not copied aspect fixtures.
use super::*;
use schema::{ApplicationScalarValueBinding, ApplicationSchema};
mod composition_refusal;
use worth_foundational::facade::{AbsenceLaw, AspectShape, WorthValue};
use worth_query_declaration::facade::{
    authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
};

trait Fixture: ApplicationSchema {}
worth_query_declaration::worth_query_entity!(Entity for S: Fixture);
worth_query_declaration::worth_query_aspect!(Publication for S: Fixture, Entity;
    identity = AspectIdentity(0x0071_9301), revision = AspectContractRevision(1));
worth_query_declaration::worth_query_aspect!(Identity for S: Fixture, Entity;
    identity = AspectIdentity(0x0071_9302), revision = AspectContractRevision(1));
#[derive(WorthValue)]
struct Payload {
    nested: Vec<Choice>,
}
#[derive(WorthValue)]
enum Choice {
    Empty,
    Selected { label: String, count: u64 },
}
#[derive(WorthValue)]
struct ChangedPayload {
    nested: Vec<ChangedChoice>,
}
#[derive(WorthValue)]
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
worth_query_declaration::worth_query_field!(Record for S: Fixture, Entity, Publication:
    Payload => Binding, read_only, no_equality);
worth_query_declaration::worth_query_field!(Sibling for S: Fixture, Entity, Publication:
    optional String => schema::StringApplicationValueBinding, read_only, no_equality);
worth_query_declaration::worth_query_field!(Key for S: Fixture, Entity, Identity:
    String => schema::StringApplicationValueBinding, read_only, no_equality);
mod twin {
    use super::*;
    worth_query_declaration::worth_query_field!(pub(super) Record for S: Fixture, Entity, Publication:
        Payload => TwinBinding, read_only, no_equality);
}
mod changed {
    use super::*;
    worth_query_declaration::worth_query_field!(pub(super) Record for S: Fixture, Entity, Publication:
        ChangedPayload => ChangedBinding, read_only, no_equality);
}
macro_rules! fixture {
    ($S:ident, $field:path) => {
        worth_query_declaration::worth_query_application_schema!(schema $S {
            owner: $S, version: (1,0), members: |schema| {
                schema.entity(Entity::reference())
                    .aspect(Entity::reference(),Identity::reference())
                    .field(Entity::reference(),Key::reference())
                    .optional_aspect(Entity::reference(),Publication::reference())
                    .field(Entity::reference(),Sibling::reference())
                    .field(Entity::reference(),<$field>::reference())
            }
        });
        impl Fixture for $S {}
    };
}
fixture!(Actual, Record);
fixture!(Changed, changed::Record);
// A valid declaration lacking the requested field, with a real remaining sibling.
worth_query_declaration::worth_query_application_schema!(schema Missing {
    owner: Missing, version: (1,0), members: |schema| {
        schema.entity(Entity::reference())
            .aspect(Entity::reference(),Identity::reference())
            .field(Entity::reference(),Key::reference())
            .optional_aspect(Entity::reference(),Publication::reference())
            .field(Entity::reference(),Sibling::reference())
    }
});
impl Fixture for Missing {}

fn installed() -> domain::WorthQueryInstalledApplicationSchema<Actual> {
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
fn query() -> CanonicalQueryBundle {
    let request = DetailQueryBuilder::new(RootEntityKey::new("Entity").unwrap())
        .project(AspectFieldSelector::new("Identity", "Key").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("Identity", "Key", "key").unwrap())
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
fn builder(
    declaration: &schema::ErasedApplicationSchemaDeclaration,
) -> WorthQueryOutputReadinessContractBuilder {
    WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        Key::reference::<Actual>(),
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
    let field = FieldExpectation::declared(
        installed.installed_declaration(),
        Record::reference::<Actual>(),
    )
    .unwrap();
    field.seal(&installed).unwrap();
    assert_eq!(field.aspect.absence(), AbsenceLaw::Optional);
    let AspectShape::Struct(shape) = field.aspect.shape() else {
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
        FieldExpectation::declared(changed.erased(), changed::Record::reference::<Changed>())
            .unwrap();
    assert_ne!(
        field.aspect, changed_field.aspect,
        "nested rename changes consumed readiness contract"
    );
    let value = Payload {
        nested: vec![Choice::Selected {
            label: "δ".into(),
            count: 17,
        }],
    };
    let encoded = Binding::encode(&value).unwrap();
    let schema::ApplicationValue::String(domain::InternedString::Raw(text)) = encoded else {
        panic!("canonical text")
    };
    assert_eq!(
        text,
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
            FieldExpectation::declared(&declaration, Record::reference::<Actual>()).unwrap_err();
        assert_eq!(
            denial.field_locus(),
            Some(&schema::ApplicationFieldBindingLocus::new(
                "Entity",
                "Publication",
                "Record"
            ))
        );
    }
    // Portable bytes agree, but the actual installed Binding TypeId differs.
    let twin = FieldExpectation::declared(
        installed.installed_declaration(),
        twin::Record::reference::<Actual>(),
    )
    .unwrap();
    let denial = twin.seal(&installed).unwrap_err();
    assert_eq!(denial.field_locus().unwrap().field(), "Record");
    assert!(denial.detail().contains("installed value binding"));
}

#[test]
fn canonical_order_and_distinct_delivery_eligibility_are_preserved() {
    let installed = installed();
    let declaration = installed.installed_declaration();
    let both = WorthQueryReadinessFieldUse::DeliveryAndEligibility;
    let first = builder(declaration)
        .field_dependency(declaration, Record::reference::<Actual>(), both)
        .unwrap()
        .field_dependency(declaration, Key::reference::<Actual>(), both)
        .unwrap()
        .build_typed::<(), (), ()>()
        .unwrap();
    let reversed = builder(declaration)
        .field_dependency(declaration, Key::reference::<Actual>(), both)
        .unwrap()
        .field_dependency(declaration, Record::reference::<Actual>(), both)
        .unwrap()
        .build_typed::<(), (), ()>()
        .unwrap();
    assert_eq!(first.fields, reversed.fields);
    assert_eq!(first.delivery_fields, reversed.delivery_fields);
    assert_eq!(
        first.definition().clone().into_portable(),
        reversed.definition().clone().into_portable()
    );
    let node = &first.definition().semantics().conditional_nodes[0];
    let ordinal = first
        .delivery_ordinal(&installed, node, Record::reference::<Actual>())
        .unwrap();
    let expected = FieldExpectation::declared(declaration, Record::reference::<Actual>())
        .unwrap()
        .dependency()
        .unwrap();
    assert_eq!(node.dependencies()[ordinal], expected);
    let distinct = builder(declaration)
        .field_dependency(
            declaration,
            Record::reference::<Actual>(),
            WorthQueryReadinessFieldUse::Delivery,
        )
        .unwrap()
        .field_dependency(
            declaration,
            Key::reference::<Actual>(),
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
        .delivery_ordinal(&installed, node, Record::reference::<Actual>())
        .unwrap();
    assert_eq!(node.dependencies()[ordinal], expected);
    assert!(distinct
        .delivery_ordinal(&installed, node, Key::reference::<Actual>())
        .is_err());
}
