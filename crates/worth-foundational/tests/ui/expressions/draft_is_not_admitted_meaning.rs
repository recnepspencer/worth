use worth_foundational::expression_api::{expressions, AdmittedExpression};

fn requires_admitted(_expression: AdmittedExpression) {}

fn main() {
    let draft = expressions().parse("1.0").unwrap();
    requires_admitted(draft);
}
