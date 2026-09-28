use std::path::PathBuf;

use crate::{
    WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileReport,
    WorthUiDslCompiler, WorthUiSealedExpression, WorthUiSealedSemanticPackage,
};

/// Projections every expression test can read: one text scalar, one boolean
/// scalar, and one collection that expressions must refuse.
pub(super) const PROJECTIONS: &str = r#"
query_scalar pulse.label { view pulse.label field label require text }
query_scalar pulse.ready { view pulse.ready field ready require boolean }
query_collection pulse.rows {
    view pulse.rows row identity field name require text
    completeness complete continuation forbidden lifecycle snapshot
}
"#;

pub(super) fn compile_expressions(declarations: &str) -> WorthUiSealedSemanticPackage {
    compile_source(&format!("{PROJECTIONS}\n{declarations}"))
        .expect("expression source should compile")
}

pub(super) fn compile_source(
    source: &str,
) -> Result<WorthUiSealedSemanticPackage, WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("main.wui", source),
    )
}

/// Every diagnostic the declarations raise, as its code and message.
pub(super) fn denials(declarations: &str) -> Vec<(WorthUiDslCompileDiagnosticCode, String)> {
    compile_source(&format!("{PROJECTIONS}\n{declarations}"))
        .expect_err("invalid expression source should fail the whole package")
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.identity().code(),
                diagnostic.message().to_owned(),
            )
        })
        .collect()
}

pub(super) fn expression<'package>(
    package: &'package WorthUiSealedSemanticPackage,
    identity: &str,
) -> &'package WorthUiSealedExpression {
    package
        .expression(identity)
        .expect("expected a sealed expression")
}
