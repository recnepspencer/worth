use worth_query_testkit_decl::fragment_auth_fixture::{
    ChangeForm, FieldsOnly, IdentityValueBinding,
};
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
    assert_eq!(clears, ["measured", "named"]);
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
fn principal_identity_rejects_a_writable_record_field() {
    let field = record_field(Fixture, Item, Metrics, "components[1]").unwrap();
    assert!(ApplicationPrincipalIdentityRequirement::<Fixture, Item, u64, IdentityValueBinding>::from_record(Fixture, field).is_err());
}

struct NoEquality;
struct RoleOwner;
struct RoleIdentity;
struct IdentityRow(u64);
struct NoEqualityBindings;
impl Fragment for NoEquality {
    const NAME: &'static str = "no_equality";
    type Bindings = NoEqualityBindings;
    type Members = (RoleOwner,);
}
impl Entity for RoleOwner {
    const NAME: &'static str = "owner";
    type Aspects = (RoleIdentity,);
    type Relations = ();
}
impl Aspect for RoleIdentity {
    const NAME: &'static str = "identity";
    type Row = IdentityRow;
}
impl BindingSet for NoEqualityBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<u64> for NoEqualityBindings {
    type Binding = IdentityValueBinding;
}
impl Record<IdentityRow> for NoEqualityBindings {
    fn record() -> impl Shape<IdentityRow, Self> {
        field("id", |row: &IdentityRow| row.0).read_only().build(
            |id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityRow(id))
            },
        )
    }
}
#[test]
fn principal_identity_requires_the_bindings_equality_posture() {
    let field = record_field(NoEquality, RoleOwner, RoleIdentity, "id").unwrap();
    assert!(ApplicationPrincipalIdentityRequirement::<Fixture, RoleOwner, u64, IdentityValueBinding>::from_record(NoEquality, field).is_err());
}
