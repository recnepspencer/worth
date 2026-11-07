//! One binding contract supplies units, validation, and a queryable scalar field.
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
worth_query_decl::facade::worth_query_value_binding!(pub(crate) WrongIdentityBinding for PositiveLength { identity: "worth.query.certification.foreign-length.v1", scalar: UInt64, unit: "Metre", frame: "worth.frames.model-local.v1", encode: PositiveLength::get, decode: PositiveLength::new, });
pub(crate) struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = Bindings;
    type Members = (Body,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.certification.binding-hostile";
    const NAME: &'static str = "Schema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub(crate) struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<PositiveLength> for Bindings {
    type Binding = WrongIdentityBinding;
}
pub(crate) struct Body;
impl Entity for Body {
    const NAME: &'static str = "Body";
    type Aspects = (Geometry,);
    type Relations = ();
}
pub(crate) struct Geometry;
impl Aspect for Geometry {
    const NAME: &'static str = "Geometry";
    type Row = GeometryValue;
}

pub(crate) struct GeometryValue {
    pub value: PositiveLength,
}
impl Record<GeometryValue> for Bindings {
    fn record() -> impl Shape<GeometryValue, Self> {
        field("value", |row: &GeometryValue| row.value)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GeometryValue { value })
            })
    }
}
