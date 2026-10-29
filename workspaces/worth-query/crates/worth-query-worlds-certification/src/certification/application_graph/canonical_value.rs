//! Canonical nested authored meaning survives the installed native checkpoint boundary.
#[path = "canonical_value/binding.rs"]
pub mod binding;
#[path = "canonical_value/host.rs"]
pub mod host;
#[path = "canonical_value/model.rs"]
pub mod model;
#[path = "canonical_value/query.rs"]
pub mod query;
#[path = "canonical_value/schema.rs"]
pub mod schema;

pub use binding::NestedBinding;
pub use model::*;
pub use worth_foundational::facade::{
    AspectValue, ValueCodecDenial, ValueDecodeBounds, ValueFieldShape, ValuePrimitive, ValueShape,
    ValueVariantShape,
};
pub use worth_query_host::facade::{
    application_installation::WorthQueryCheckpointCapturePolicy,
    declaration::application_schema::{
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
        ApplicationValueDecodeDenial,
    },
};

pub fn assert_installed_contract(application: &host::Host) {
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

pub fn expected_shape() -> ValueShape {
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
