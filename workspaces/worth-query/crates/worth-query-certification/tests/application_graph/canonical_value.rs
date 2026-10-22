//! Canonical nested authored meaning survives the installed native checkpoint boundary.
#[path = "canonical_value/binding.rs"]
mod binding;
#[path = "canonical_value/host.rs"]
mod host;
#[path = "canonical_value/model.rs"]
mod model;
#[path = "canonical_value/query.rs"]
mod query;
#[path = "canonical_value/schema.rs"]
mod schema;

use binding::NestedBinding;
use model::*;
use worth_foundational::facade::{
    AspectValue, ValueCodecDenial, ValueDecodeBounds, ValueFieldShape, ValuePrimitive, ValueShape,
    ValueVariantShape,
};
use worth_query_host::facade::{
    application_installation::WorthQueryCheckpointCapturePolicy,
    declaration::application_schema::{
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
        ApplicationValueDecodeDenial,
    },
};

#[test]
fn nondefault_nested_value_uses_derived_binding_catalog_and_native_checkpoint() {
    let authored = authored_value();
    assert_eq!(
        NestedBinding::encode(&authored).unwrap(),
        AspectValue::String(AUTHORED_WIRE.into())
    );
    assert_eq!(
        NestedBinding::decode(&AspectValue::String(AUTHORED_WIRE.into())).unwrap(),
        authored
    );

    let application =
        host::install(authored.clone()).expect("typed source seed encodes through its binding");
    assert_installed_contract(&application);
    assert_eq!(host::read(&application), authored);
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .expect("native source checkpoint captures")
        })
        .expect("host capture phase opens");
    drop(application);

    let restored = host::restore(checkpoint);
    assert_installed_contract(&restored);
    assert_eq!(host::read(&restored), authored);
}

#[test]
fn ordinary_nested_storage_accepts_values_beyond_former_representation_ceilings() {
    let mut authored = authored_value();
    authored.children[0].tag.0 = "λ".repeat(257);
    authored.children = vec![authored.children[0].clone(); 17];
    let encoded = NestedBinding::encode(&authored).unwrap();
    assert_eq!(NestedBinding::decode(&encoded).unwrap(), authored);
    let application = host::install(authored.clone())
        .expect("ordinary canonical storage requires no guessed representation limits");
    assert_eq!(host::read(&application), authored);
}

#[test]
fn unrestricted_binding_still_refuses_a_field_outside_its_declared_grammar() {
    let malformed = AUTHORED_WIRE.replacen("\"title\":", "\"unexpected\":", 1);
    assert!(matches!(
        NestedBinding::decode(&AspectValue::String(malformed.into())),
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::Malformed,
            ..
        })
    ));
}

fn assert_installed_contract(application: &host::Host) {
    let installed = application
        .installed_schema()
        .value_bindings()
        .field("Document", "DocumentFacts", "DocumentValueField")
        .expect("exact installed field retains its value binding");
    let contract = installed
        .canonical_contract()
        .expect("generated canonical contract installs");
    assert_eq!(contract.wire_protocol(), "worth.canonical-json.v1");
    assert_eq!(contract.bounds(), ValueDecodeBounds::unrestricted());
    assert_eq!(contract.shape(), &expected_shape());
    assert_eq!(
        installed.encode(&authored_value()).unwrap(),
        AspectValue::String(AUTHORED_WIRE.into())
    );
}

fn expected_shape() -> ValueShape {
    let primitive = ValueShape::Primitive;
    let child = ValueShape::Struct(vec![
        ValueFieldShape::new("tag", primitive(ValuePrimitive::String)),
        ValueFieldShape::new("active", primitive(ValuePrimitive::Bool)),
        ValueFieldShape::new("rank", primitive(ValuePrimitive::I32)),
    ]);
    ValueShape::Struct(vec![
        ValueFieldShape::new("title", primitive(ValuePrimitive::String)),
        ValueFieldShape::new(
            "selection",
            ValueShape::Enum(vec![
                ValueVariantShape::unit("Idle"),
                ValueVariantShape::newtype("chosen", child.clone()),
            ]),
        ),
        ValueFieldShape::new("children", ValueShape::Sequence(Box::new(child))),
        ValueFieldShape::new(
            "notes",
            ValueShape::Optional(Box::new(primitive(ValuePrimitive::String))),
        ),
    ])
}
