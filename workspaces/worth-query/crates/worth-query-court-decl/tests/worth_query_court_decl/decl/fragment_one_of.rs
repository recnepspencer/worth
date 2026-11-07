use worth_query_testkit_decl::fragment_auth_fixture::{ChangeForm, FieldsOnly};
use worth_query_testkit_decl::fragment_fixture::*;

#[test]
fn whole_record_writes_derive_sibling_clears_but_single_fields_do_not() {
    let schema = declare(Fixture).unwrap();
    let clears: Vec<_> = schema
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationProgram {
                operation,
                target: ApplicationOperationProgramTarget::RemoveAspect { aspect, .. },
            } if operation == ChangeForm::reference().name() => Some(aspect.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        clears,
        [
            "a517cf620c4627e67",
            "a83ebc555df84f92e",
            "ac9a7ff83b921347e"
        ]
    );
    assert!(!schema.erased().members().iter().any(|member| matches!(member,
        ApplicationSchemaMember::OperationProgram { operation, target: ApplicationOperationProgramTarget::RemoveAspect { .. } }
        if operation == FieldsOnly::reference().name())));
}

struct AmbiguousRows;
impl Fragment for AmbiguousRows {
    const NAME: &'static str = "ambiguous_rows";
    type Bindings = Bindings;
    type Members = (AmbiguousEntity,);
}
impl ApplicationSchema for AmbiguousRows {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
struct AmbiguousEntity;
impl Entity for AmbiguousEntity {
    const NAME: &'static str = "item";
    type Aspects = (Exclusive<Ambiguous>,);
    type Relations = ();
}
struct Ambiguous;
impl OneOf for Ambiguous {
    type Row = Common;
    type Cases = (First, Second);
}
struct First;
impl Aspect for First {
    const NAME: &'static str = "first";
    type Row = Caption;
}
struct Second;
impl Aspect for Second {
    const NAME: &'static str = "second";
    type Row = Caption;
}
enum Common {
    Value(Caption),
}
impl From<Caption> for Common {
    fn from(value: Caption) -> Self {
        Self::Value(value)
    }
}

#[test]
fn one_from_impl_cannot_make_two_same_row_cases_unambiguous() {
    let Common::Value(value) = Common::from(Caption {
        text: "sample".to_owned(),
    });
    assert_eq!(value.text, "sample");
    assert!(matches!(
        declare(AmbiguousRows),
        Err(ApplicationSchemaDeclarationDenial::DuplicateOneOfRow { .. })
    ));
}

#[test]
fn internal_presence_columns_are_not_user_field_handles() {
    let selected: Result<RecordField<_, _, bool>, _> =
        record_field(Fixture, Item, Noted, "__record_presence").resolve();
    assert!(selected.is_err());
    let declaration = declare(Fixture).unwrap();
    let selection: FieldSelection<_, _, _, bool> =
        select_field(Fixture, Item, Noted, "__record_presence");
    assert!(selection
        .installed(declaration.member_provenance().fragment_catalog())
        .is_err());
}
