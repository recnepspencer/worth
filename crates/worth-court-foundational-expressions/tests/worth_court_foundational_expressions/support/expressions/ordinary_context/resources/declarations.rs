// Resource ceilings: every phase denies before unbounded work, and a draft
// read under wide ceilings is re-checked against the admitting profile.
pub(crate) use {
    worth_foundational::expression_api::expressions,
    worth_foundational::expression_api::ExpressionBuilder,
    worth_foundational::expression_api::ExpressionDenial,
    worth_foundational::expression_api::ExpressionDenialDetail,
    worth_foundational::expression_api::ExpressionFunctionCatalog,
    worth_foundational::expression_api::ExpressionProfile,
    worth_foundational::expression_api::ExpressionResource,
    worth_foundational::expression_api::ExpressionSchema,
    worth_foundational::expression_api::ExpressionType,
};

pub(crate) use {
    crate::expressions::ordinary_context::draft,
    crate::expressions::ordinary_context::empty_catalog,
    crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::schema,
};

pub(crate) fn exceeded(denial: &ExpressionDenial) -> ExpressionResource {
    match denial.detail() {
        worth_foundational::expression_api::ExpressionDenialDetail::ResourceExceeded {
            resource,
            ..
        } => *resource,
        other => panic!("expected a resource denial, found {other:?}"),
    }
}

pub(crate) fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    worth_foundational::expression_api::ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("narrowing lowers a ceiling")
}
