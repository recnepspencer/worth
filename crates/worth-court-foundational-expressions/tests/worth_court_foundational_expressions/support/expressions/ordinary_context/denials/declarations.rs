// Exact denial families for builtin, constructor, binder, record, and
// decoder contracts, plus the admission meter and identity invariants that
// guard them.
pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::Dynamic, worth_foundational::expression_api::ExpressionBuilder, worth_foundational::expression_api::ExpressionDenial, worth_foundational::expression_api::ExpressionDenialDetail, worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionFunctionCatalog, worth_foundational::expression_api::ExpressionFunctionDeclaration, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionResource, worth_foundational::expression_api::ExpressionSchema, worth_foundational::expression_api::ExpressionType, worth_foundational::expression_api::ExpressionTypeArgument, worth_foundational::expression_api::ExpressionTypeName, worth_foundational::expression_api::IntegerType, worth_foundational::expression_api::Term};

pub(crate) use {crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::denied_family, crate::expressions::ordinary_context::draft, crate::expressions::ordinary_context::empty_catalog, crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::schema};

pub(crate) fn exceeded<T: std::fmt::Debug>(
    result: Result<T, ExpressionDenial>,
) -> ExpressionResource {
    let denial = result.expect_err("a resource ceiling denies");
    match denial.detail() {
        worth_foundational::expression_api::ExpressionDenialDetail::ResourceExceeded { resource, .. } => *resource,
        other => panic!("expected a resource denial, found {other:?}"),
    }
}

pub(crate) fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    worth_foundational::expression_api::ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("narrowing lowers a ceiling")
}

pub(crate) fn area() -> ExpressionFunctionDeclaration {
    function(
        "geometry::area",
        &[
            ("w", worth_foundational::expression_api::ExpressionType::Float64),
            ("h", worth_foundational::expression_api::ExpressionType::Float64),
        ],
        worth_foundational::expression_api::ExpressionType::Float64,
        "w * h",
    )
}

pub(crate) fn catalog(
    schema: &ExpressionSchema,
    functions: impl IntoIterator<Item = ExpressionFunctionDeclaration>,
) -> ExpressionFunctionCatalog {
    let mut builder = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(schema, worth_foundational::expression_api::ExpressionProfile::interactive());
    for declaration in functions {
        builder
            .install(declaration)
            .expect("fixture functions install");
    }
    builder.build()
}