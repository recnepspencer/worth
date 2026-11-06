//! Root-specific contribution ownership composes one reusable record fragment.
pub use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};

pub struct Model;
impl Fragment for Model {
    const NAME: &'static str = "entries";
    type Bindings = Bindings;
    type Members = (Entry,);
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}
pub struct Entry;
impl Entity for Entry {
    const NAME: &'static str = "entry";
    type Aspects = (Payload, Option<OtherPayload>);
    type Relations = ();
}
pub struct Payload;
impl Aspect for Payload {
    const NAME: &'static str = "payload";
    type Row = Details;
}
pub struct OtherPayload;
impl Aspect for OtherPayload {
    const NAME: &'static str = "other";
    type Row = Details;
}
pub struct Details {
    pub title: Option<String>,
}
impl Record<Details> for Bindings {
    fn record() -> impl Shape<Details, Self> {
        field("title", |row: &Details| row.title.clone()).build(
            |title, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Details { title })
            },
        )
    }
}
worth_query_decl::facade::worth_query_structured_value_binding!(pub Input for () { identity: "worth.rust.unit" });
pub struct Edit;
impl<S> ApplicationOperationMarkerIdentity<S> for Edit {
    type InputBinding = Input;
    const IDENTIFIER: &'static str = "edit";
}
impl OperationCreates<Edit> for Entry {}
impl<S: ApplicationSchema> Registration<S> for Edit {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let operation = ApplicationOperationRef::<S, Self, ()>::from_declaration();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_record(operation, Model, Entry, Payload)
            .operation_write_record(operation, Model, Entry, Payload)
            .operation_create(operation, entity(Model, Entry)))
    }
}
pub struct Common;
impl<S: ApplicationSchema> ApplicationSchemaContribution<S> for Common {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("shared");
    type Members = (Model, Edit);
}
pub struct Outside;
impl<S: ApplicationSchema> Registration<S> for Outside {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.operation_read_record(
            ApplicationOperationRef::<S, Edit, ()>::from_declaration(),
            Model,
            Entry,
            OtherPayload,
        ))
    }
}
pub struct Composed;
impl Fragment for Composed {
    const NAME: &'static str = "composed";
    type Bindings = NoBindings;
    type Members = (Common,);
}
impl ApplicationSchema for Composed {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct Duplicate;
impl Fragment for Duplicate {
    const NAME: &'static str = "duplicate";
    type Bindings = NoBindings;
    type Members = (Common, Common);
}
impl ApplicationSchema for Duplicate {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct Incomplete;
impl Fragment for Incomplete {
    const NAME: &'static str = "incomplete";
    type Bindings = NoBindings;
    type Members = (Common, Outside);
}
impl ApplicationSchema for Incomplete {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
