//! Explicit whole-aspect absence through the consumer declaration facade.
// compiled-example: begin
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct OptionalRecords;
impl Fragment for OptionalRecords {
    const NAME: &'static str = "optional_records";
    type Bindings = OptionalBindings;
    type Members = (Component,);
}
pub struct OptionalBindings;
impl BindingSet for OptionalBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<ComponentIdentifier> for OptionalBindings {
    type Binding = ComponentIdentifierBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl Scalar<String> for OptionalBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub struct Component;
impl Entity for Component {
    const NAME: &'static str = "Component";
    type Aspects = (Identity, Option<Annotation>);
    type Relations = ();
}
impl Component {
    pub fn reference<Schema: ApplicationSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(OptionalRecords, Self)
    }
}
pub struct Identity;
impl Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = ComponentIdentity;
}
pub struct Annotation;
impl Aspect for Annotation {
    const NAME: &'static str = "Annotation";
    type Row = ComponentAnnotation;
}
pub struct ComponentIdentity {
    pub identifier: Option<ComponentIdentifier>,
}
pub struct ComponentAnnotation {
    pub text: String,
}
impl fragment::Record<ComponentIdentity> for OptionalBindings {
    fn record() -> impl Shape<ComponentIdentity, Self> {
        field("Identifier", |row: &ComponentIdentity| {
            row.identifier.clone()
        })
        .build(|identifier, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ComponentIdentity { identifier })
        })
    }
}
impl fragment::Record<ComponentAnnotation> for OptionalBindings {
    fn record() -> impl Shape<ComponentAnnotation, Self> {
        field("Text", |row: &ComponentAnnotation| row.text.clone()).build(
            |text, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ComponentAnnotation { text })
            },
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentIdentifier(String);
impl ComponentIdentifier {
    fn text(&self) -> worth_foundational::facade::InternedString {
        self.0.clone().into()
    }
    fn checked(value: worth_foundational::facade::InternedString) -> Option<Self> {
        match value {
            worth_foundational::facade::InternedString::Raw(text) => Some(Self(text)),
            worth_foundational::facade::InternedString::Symbol(_) => None,
        }
    }
}
impl From<String> for ComponentIdentifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}
worth_query_decl::facade::worth_query_value_binding!(pub ComponentIdentifierBinding for ComponentIdentifier { identity: "worth.query.fixture.component-identifier.v1", scalar: String, encode: ComponentIdentifier::text, decode: ComponentIdentifier::checked, });
pub type Identifier =
    FieldSelection<OptionalRecords, Component, Identity, Option<ComponentIdentifier>>;
pub type Text = FieldSelection<OptionalRecords, Component, Annotation, String>;
// compiled-example: end
