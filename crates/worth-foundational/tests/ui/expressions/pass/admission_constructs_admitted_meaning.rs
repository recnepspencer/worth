use worth_foundational::expression_api::{
    expressions, AdmittedExpression, ExpressionFunctionCatalog, ExpressionProfile, ExpressionSchema,
};

fn requires_admitted(expression: AdmittedExpression) -> bool {
    expression.slots().len() == 0
}

fn main() {
    let schema = ExpressionSchema::builder().build();
    let catalog = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    let admitted = expressions()
        .parse("1.0 + 2.0")
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .unwrap();
    assert!(requires_admitted(admitted));
}
