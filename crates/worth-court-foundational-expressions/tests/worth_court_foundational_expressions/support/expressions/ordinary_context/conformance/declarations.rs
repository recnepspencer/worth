// V1 grammar conformance: every production parses to the documented shape,
// and syntax outside the grammar is denied rather than implementation-defined.
pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::ExpressionDenialDetail, worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionOccurrence, worth_foundational::expression_api::ExpressionType, worth_foundational::expression_api::IntegerType, worth_foundational::expression_api::SourceOrigin, worth_foundational::expression_api::SyntaxDenial};

pub(crate) use {crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::admitted_type, crate::expressions::ordinary_context::denied_family, crate::expressions::ordinary_context::length, crate::expressions::ordinary_context::nominal};

pub(crate) fn syntax_denial(source: &str) -> SyntaxDenial {
    match expressions().parse(source) {
        Ok(_) => panic!("{source} parsed"),
        Err(denial) => match denial.detail() {
            worth_foundational::expression_api::ExpressionDenialDetail::Syntax(syntax) => syntax.clone(),
            other => panic!("{source}: expected a syntax denial, found {other:?}"),
        },
    }
}