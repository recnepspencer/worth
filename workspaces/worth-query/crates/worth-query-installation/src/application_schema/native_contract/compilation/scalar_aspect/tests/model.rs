//! One actual scalar value lowers to an ordinary record column.
use worth_query_declaration::facade::application_schema::{fragment as records, *};
pub(super) struct FlatSchema;
impl records::Fragment for FlatSchema {
    const NAME: &'static str = "ScalarCompilerFixture";
    type Bindings = Bindings;
    type Members = (Record, Read);
}
impl ApplicationSchema for FlatSchema {
    const NAME: &'static str = <Self as records::Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        records::declare(Self)
    }
}
pub(super) struct Bindings;
impl records::BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl records::Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
pub(super) struct Record;
impl records::Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = (Facts,);
    type Relations = ();
}
pub(super) struct Facts;
impl records::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = u64;
}
pub(super) fn value() -> records::FieldSelection<FlatSchema, Record, Facts, u64> {
    records::select_field(FlatSchema, Record, Facts, "value")
}
worth_query_declaration::worth_query_structured_value_binding!(pub(super) InputBinding for () { identity: "worth.rust.unit" });
worth_query_declaration::worth_query_operation!(pub(super) Read for FlatSchema, input InputBinding);
impl records::Registration<FlatSchema> for Read {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<FlatSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<FlatSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(Self::reference(), value()))
    }
}
