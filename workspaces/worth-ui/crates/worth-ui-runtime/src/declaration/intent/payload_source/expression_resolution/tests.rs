//! The payload admission matrix: a text field reads only a `derived text`, an
//! unsigned 64-bit field only a `derived integer`, a Boolean field only a
//! condition, and a selection field no expression. Every other pairing is a
//! typed denial, never a conversion.

use std::path::PathBuf;

use worth_ui_dsl::{
    WorthUiAuthoredSourceInput, WorthUiDslCompiler, WorthUiExpressionResultType as Result,
    WorthUiExpressionRole as Role,
};
use worth_ui_query_binding::WorthUiQueryBindingPlan;

use super::{resolve_condition, resolve_derived};
use crate::capability::{
    UiIntentBoolean, UiIntentPayload, UiIntentPayloadField, UiIntentPayloadFieldDescriptor,
    UiIntentPayloadFieldKind as Kind, UiIntentPayloadFieldSet, UiIntentPayloadProjection,
    UiIntentPayloadProjectionViolation, UiIntentSchema, UiIntentSelection, UiIntentText,
    UiIntentUnsigned64,
};
use crate::declaration::intent::{
    UiIntentApplicationFact, UiIntentApplicationFactPlan, UiIntentCatalogPreparationDenial,
    UiResolvedIntentPayloadSource,
};
use crate::runtime::expression::{UiExpressionCatalog, WorthUiAuthoredExpressionMaterial};

const DECLARATION: &str = "test.payload.route";

const EXPRESSIONS: &str = r#"
condition ex.ready { operand r application-boolean app.ready; when (r) }
derived ex.label { operand n application-text app.name; result text; value (n) }
derived ex.tone { operand n application-text app.name; result token; value (n == "x" ? "a" : n) }
derived ex.count {
    operand c application-unsigned64 app.count;
    result integer;
    value (exact_cast<Int64>(c) + 1)
}
derived ex.ratio {
    operand n application-text app.name;
    result decimal;
    value (n == "x" ? decimal("1.5") : decimal("2.5"))
}
"#;

struct Payload;

impl UiIntentPayload for Payload {
    const SCHEMA: UiIntentSchema = UiIntentSchema::stable("test.payload.matrix", 1);
    const FIELDS: UiIntentPayloadFieldSet = UiIntentPayloadFieldSet::EMPTY;
    fn project(
        _: &mut UiIntentPayloadProjection<Self>,
    ) -> core::result::Result<Self, UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

fn fields() -> [UiIntentPayloadFieldDescriptor; 4] {
    [
        UiIntentPayloadField::<Payload, UiIntentText>::text(0, "text", 32).descriptor(),
        UiIntentPayloadField::<Payload, UiIntentBoolean>::boolean(1, "boolean").descriptor(),
        UiIntentPayloadField::<Payload, UiIntentUnsigned64>::unsigned64(2, "unsigned").descriptor(),
        UiIntentPayloadField::<Payload, UiIntentSelection>::selection(3, "selection").descriptor(),
    ]
}

fn catalog() -> UiExpressionCatalog {
    let package = WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("main.wui", EXPRESSIONS),
    )
    .expect("the matrix expressions seal");
    let mut facts = UiIntentApplicationFactPlan::default();
    facts
        .register_boolean(UiIntentApplicationFact::boolean("app.ready").unwrap(), true)
        .unwrap();
    facts
        .register_text(UiIntentApplicationFact::text("app.name", 16).unwrap(), "x")
        .unwrap();
    facts
        .register_unsigned64(UiIntentApplicationFact::unsigned64("app.count").unwrap(), 1)
        .unwrap();
    UiExpressionCatalog::prepare(
        &WorthUiAuthoredExpressionMaterial::from_package(&package),
        &[],
        &WorthUiQueryBindingPlan::default(),
        &facts,
    )
    .expect("the matrix expressions install")
}

fn mismatch(field: UiIntentPayloadFieldDescriptor, expression: &str, role: Role) -> Denial {
    Denial::PayloadExpressionKindMismatch {
        declaration: DECLARATION.into(),
        field: field.stable_name().into(),
        expression: expression.into(),
        field_kind: field.kind(),
        role,
    }
}

type Denial = UiIntentCatalogPreparationDenial;

#[test]
fn a_derived_value_fits_only_the_field_kind_that_carries_its_result_type() {
    let catalog = catalog();
    for field in fields() {
        for (expression, result) in [
            ("ex.label", Result::Text),
            ("ex.tone", Result::Token),
            ("ex.count", Result::Integer),
            ("ex.ratio", Result::Decimal),
        ] {
            let resolved = resolve_derived(DECLARATION, field, expression, &catalog);
            let source = match (field.kind(), result, &resolved) {
                (
                    Kind::Text,
                    Result::Text,
                    Ok(UiResolvedIntentPayloadSource::DerivedText(source)),
                )
                | (
                    Kind::Unsigned64,
                    Result::Integer,
                    Ok(UiResolvedIntentPayloadSource::DerivedInteger(source)),
                ) => Some(source),
                (Kind::Text, Result::Text, _) | (Kind::Unsigned64, Result::Integer, _) => {
                    panic!(
                        "a {:?} field reads a derived {result:?}: {resolved:?}",
                        field.kind()
                    )
                }
                _ => None,
            };
            if let Some(source) = source {
                assert_eq!(source.identity(), expression);
                assert_eq!(source.slot(), catalog.slot_of(expression).unwrap());
            } else {
                assert_eq!(
                    resolved,
                    Err(mismatch(field, expression, Role::Derived(result))),
                    "{:?} field, derived {result:?}",
                    field.kind()
                );
            }
        }
    }
}

#[test]
fn only_a_boolean_field_reads_a_condition() {
    let catalog = catalog();
    for field in fields() {
        let resolved = resolve_condition(DECLARATION, field, "ex.ready", &catalog);
        if field.kind() == Kind::Boolean {
            let Ok(UiResolvedIntentPayloadSource::Condition(source)) = resolved else {
                panic!("a Boolean field reads a condition: {resolved:?}");
            };
            assert_eq!(source.identity(), "ex.ready");
        } else {
            assert_eq!(
                resolved,
                Err(mismatch(field, "ex.ready", Role::Condition)),
                "{:?} field",
                field.kind()
            );
        }
    }
}

#[test]
fn a_source_in_the_other_role_is_a_kind_mismatch_never_a_conversion() {
    let catalog = catalog();
    let [text, boolean, ..] = fields();
    assert_eq!(
        resolve_derived(DECLARATION, boolean, "ex.ready", &catalog),
        Err(mismatch(boolean, "ex.ready", Role::Condition)),
        "a condition read as a derived value keeps its own role"
    );
    assert_eq!(
        resolve_condition(DECLARATION, text, "ex.label", &catalog),
        Err(mismatch(text, "ex.label", Role::Derived(Result::Text))),
        "a derived text read as a condition keeps its own role"
    );
}

#[test]
fn an_expression_the_catalog_does_not_install_is_unknown() {
    let catalog = catalog();
    let [text, boolean, ..] = fields();
    for (resolved, field) in [
        (
            resolve_derived(DECLARATION, text, "ex.missing", &catalog),
            text,
        ),
        (
            resolve_condition(DECLARATION, boolean, "ex.missing", &catalog),
            boolean,
        ),
    ] {
        assert_eq!(
            resolved,
            Err(Denial::UnknownPayloadExpression {
                declaration: DECLARATION.into(),
                field: field.stable_name().into(),
                expression: "ex.missing".into(),
            })
        );
    }
}
