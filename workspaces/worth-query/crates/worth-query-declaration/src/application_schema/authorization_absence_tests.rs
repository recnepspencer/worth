//! End to end through real fragment lowering: a deny condition on a required
//! column of an `Option<A>` aspect or of an `Exclusive<..>` case aspect is
//! refused at build, because the aspect itself can be absent.
use std::convert::Infallible;

use super::fragment::*;
use super::*;

struct Suite;
impl Fragment for Suite {
    const NAME: &'static str = "absence_suite";
    type Bindings = Bindings;
    type Members = (Item,);
}
impl ApplicationSchema for Suite {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}

struct Item;
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (Keyed, Option<Label>, Exclusive<Form>);
    type Relations = ();
}
struct Keyed;
impl Aspect for Keyed {
    const NAME: &'static str = "keyed";
    type Row = Text;
}
struct Label;
impl Aspect for Label {
    const NAME: &'static str = "label";
    type Row = Text;
}
struct Form;
impl OneOf for Form {
    type Row = FormValue;
    type Cases = (Named, Coded);
}
struct Named;
impl Aspect for Named {
    const NAME: &'static str = "named";
    type Row = Text;
}
struct Coded;
impl Aspect for Coded {
    const NAME: &'static str = "coded";
    type Row = Code;
}

struct Text {
    text: String,
}
struct Code {
    code: u64,
}
enum FormValue {
    Named(Text),
    Coded(Code),
}
impl From<Text> for FormValue {
    fn from(value: Text) -> Self {
        Self::Named(value)
    }
}
impl From<Code> for FormValue {
    fn from(value: Code) -> Self {
        Self::Coded(value)
    }
}

struct Bindings;
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
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl Record<Text> for Bindings {
    fn record() -> impl Shape<Text, Self> {
        field("text", |row: &Text| row.text.clone())
            .equality_queryable(true)
            .build(|text, _: &mut ValueDecodeAdmission<'_>| Ok::<_, Infallible>(Text { text }))
    }
}
impl Record<Code> for Bindings {
    fn record() -> impl Shape<Code, Self> {
        field("code", |row: &Code| row.code)
            .equality_queryable(true)
            .build(|code, _: &mut ValueDecodeAdmission<'_>| Ok::<_, Infallible>(Code { code }))
    }
}

/// Builds the schema with one deny path over `field`.
fn with_deny<A, T>(
    field: FieldSelection<Suite, Item, A, T>,
    value: ApplicationEncodedScalarValue<
        <FieldSelection<Suite, Item, A, T> as RecordSelection>::Binding,
    >,
) -> Result<ApplicationSchemaDeclaration<Suite>, ApplicationSchemaDeclarationDenial>
where
    FieldSelection<Suite, Item, A, T>: RecordSelection<Entity = Item>,
{
    let item: ApplicationEntityRef<Suite, Item> = entity(Suite, Item);
    let deny = ApplicationAuthorizationPathBuilder::from_principal(item)
        .where_equal(field, value)?
        .deny(item);
    let allow = ApplicationAuthorizationPathBuilder::from_principal(item).allow(item);
    let ability =
        ApplicationAbilityRef::<Suite, (), Item>::from_schema_identifiers("Archive", "item");
    let policy = ApplicationPolicyRef::<Suite, ()>::from_schema_identifier("Access");
    declaration_builder(Suite)?
        .ability(ability)
        .policy(policy)
        .ability_policy(ability, policy, [allow, deny])
        .build()
}

fn refused_condition(
    result: Result<ApplicationSchemaDeclaration<Suite>, ApplicationSchemaDeclarationDenial>,
) -> Option<String> {
    match result {
        Err(ApplicationSchemaDeclarationDenial::InvalidAuthorizationRule(
            ApplicationAuthorizationRuleDenial::DenyConditionOnOptionalField { path, condition },
        )) => {
            assert_eq!(path, "item");
            Some(condition)
        }
        _ => None,
    }
}

fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    match ApplicationEncodedScalarValue::try_new(value.to_owned()) {
        Ok(value) => value,
        Err(denial) => panic!("fixture text encodes: {denial:?}"),
    }
}

#[test]
fn deny_on_a_required_column_of_an_option_aspect_is_refused() {
    let condition = refused_condition(with_deny(
        record_field::<_, _, _, String, _, _>(Suite, Item, Label, "text"),
        text("x"),
    ));
    assert!(
        condition.as_deref().is_some_and(|condition| {
            condition.starts_with("item.")
                && condition.ends_with(".text (its aspect can be absent)")
        }),
        "{condition:?}"
    );
}

#[test]
fn deny_on_a_required_column_of_an_exclusive_case_aspect_is_refused() {
    let named = refused_condition(with_deny(
        record_field::<_, _, _, String, _, _>(Suite, Item, Named, "text"),
        text("x"),
    ));
    assert!(
        named
            .as_deref()
            .is_some_and(|c| c.ends_with(".text (its aspect can be absent)")),
        "{named:?}"
    );
    let code = match ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(7) {
        Ok(code) => code,
        Err(denial) => panic!("fixture code encodes: {denial:?}"),
    };
    let coded = refused_condition(with_deny(
        record_field::<_, _, _, u64, _, _>(Suite, Item, Coded, "code"),
        code,
    ));
    assert!(
        coded
            .as_deref()
            .is_some_and(|c| c.ends_with(".code (its aspect can be absent)")),
        "{coded:?}"
    );
}

#[test]
fn deny_on_a_column_of_an_always_present_aspect_is_not_refused_for_absence() {
    // The suite binds no principal, so closure still refuses the policy, but
    // not because the deny could read an absent value.
    let result = with_deny(
        record_field::<_, _, _, String, _, _>(Suite, Item, Keyed, "text"),
        text("x"),
    );
    assert_eq!(refused_condition(result.clone()), None);
    assert_eq!(
        result.err(),
        Some(ApplicationSchemaDeclarationDenial::InvalidAbilityPolicy)
    );
}
