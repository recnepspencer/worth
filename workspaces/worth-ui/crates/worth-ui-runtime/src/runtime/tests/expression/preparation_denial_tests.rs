use super::session_fixture::prepare;
use crate::facade::expression::UiExpressionCatalogPreparationDenial;
use crate::facade::lifecycle::WorthUiApplicationPreparationDenial;

const UNREGISTERED_TEXT: &str = "query_scalar app.pulse { view app.pulse field status require text }\ncondition ex.pulse { operand p query-scalar app.pulse; when (p == \"live\") }";
const BOOLEAN_SCALAR: &str = "query_scalar app.flag { view app.flag field status require boolean }\ncondition ex.flag { operand f query-scalar app.flag; when (f) }";
const UNKNOWN_FACT: &str =
    "condition ex.unknown { operand r application-boolean app.unregistered; when (r) }";

fn expression_denial(declarations: &str) -> UiExpressionCatalogPreparationDenial {
    match prepare(declarations) {
        Err(WorthUiApplicationPreparationDenial::ExpressionCatalog(denial)) => *denial,
        Err(other) => panic!("expected an expression catalog denial, got {other:?}"),
        Ok(_) => panic!("the application must not prepare"),
    }
}

#[test]
fn an_unregistered_application_fact_denies_the_whole_application() {
    assert_eq!(
        expression_denial(UNKNOWN_FACT),
        UiExpressionCatalogPreparationDenial::UnknownApplicationFact {
            expression: "ex.unknown".into(),
            operand: "r".into(),
            fact: "app.unregistered".into(),
        }
    );
}

#[test]
fn a_query_scalar_the_binding_plan_does_not_carry_denies_the_application() {
    assert_eq!(
        expression_denial(UNREGISTERED_TEXT),
        UiExpressionCatalogPreparationDenial::UnboundProjection {
            expression: "ex.pulse".into(),
            operand: "p".into(),
            view: "app.pulse".into(),
        }
    );
}

#[test]
fn a_boolean_query_scalar_is_denied_until_its_binding_path_exists() {
    assert_eq!(
        expression_denial(BOOLEAN_SCALAR),
        UiExpressionCatalogPreparationDenial::UnsupportedQueryScalarType {
            expression: "ex.flag".into(),
            operand: "f".into(),
            projection: "app.flag".into(),
        }
    );
}
