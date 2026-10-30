// Bounded suspension and nested budgets: where slices end never changes an
// evaluation's outcome, reads, or logical cost, and no slice, call, or
// supplied profile resets or widens the admitted limits.
pub(crate) use {
    worth_foundational::expression_api::expressions,
    worth_foundational::expression_api::ExpressionCost,
    worth_foundational::expression_api::ExpressionDenialDetail,
    worth_foundational::expression_api::ExpressionEvaluation,
    worth_foundational::expression_api::ExpressionFunctionCatalog,
    worth_foundational::expression_api::ExpressionInputs,
    worth_foundational::expression_api::ExpressionProfile,
    worth_foundational::expression_api::ExpressionResource,
    worth_foundational::expression_api::ExpressionSchema,
    worth_foundational::expression_api::ExpressionStep,
    worth_foundational::expression_api::ExpressionType,
    worth_foundational::expression_api::ExpressionValue,
    worth_foundational::expression_api::MAX_SLICE_QUANTUM,
};

pub(crate) use {
    crate::expressions::ordinary_context::admit,
    crate::expressions::ordinary_context::empty_catalog,
    crate::expressions::ordinary_context::evaluate, crate::expressions::ordinary_context::function,
    crate::expressions::ordinary_context::inputs, crate::expressions::ordinary_context::schema,
};

pub(crate) const CORPUS: [&str; 16] = [
    "width * 2.0 - depth",
    r#""a long string that spans several words" == "a long string that spans several wordz""#,
    "[[1, 2], [3, 4]] == [[1, 2], [3, 5]]",
    "min([5, 3, 9, 3, 1]) ?? 0",
    "sum([1.5, 2.5, 3.0, 4.0])",
    r#"{"delta": 4, "alpha": 1, "charlie": 3, "bravo": 2, "echo": 5}"#,
    r#"{"a": 1, "c": 2, "b": 3, "a": 4}"#,
    r#"get({"delta": 4, "alpha": 1, "charlie": 3}, "charlie")"#,
    r#"entries({"b": 2, "a": 1}).map(e, e.value)"#,
    r#"length("ünïcödé text that runs past one word")"#,
    r#"slice("ünïcödé text that runs past one word", 3, 30)"#,
    r#"contains("abababababababababac", "ababac")"#,
    r#"ends_with("engineering drawing", "drawing")"#,
    "members.filter(m, m.material == Material::Steel).map(m, m.thickness * 2.0)",
    "clamp(count, 9, 1)",
    r#"decimal_div(decimal("10"), decimal("3"), 6, Rounding::NearestEven)"#,
];

pub(crate) const QUANTA: [u64; 6] = [1, 2, 3, 7, 64, MAX_SLICE_QUANTUM];

pub(crate) const LOGICAL: [ExpressionResource; 6] = [
    worth_foundational::expression_api::ExpressionResource::SemanticWork,
    worth_foundational::expression_api::ExpressionResource::VisitedElements,
    worth_foundational::expression_api::ExpressionResource::InputBytes,
    worth_foundational::expression_api::ExpressionResource::OutputBytes,
    worth_foundational::expression_api::ExpressionResource::ScratchBytes,
    worth_foundational::expression_api::ExpressionResource::RetainedConsumptionBytes,
];

/// Every counter except the slice count, which depends on the quantum.
pub(crate) fn counters(cost: &ExpressionCost) -> Vec<u64> {
    let mut counters: Vec<u64> = LOGICAL
        .iter()
        .map(|resource| cost.used(*resource))
        .collect();
    counters.extend([
        cost.reads(),
        cost.calls(),
        cost.comparisons(),
        cost.allocations(),
        cost.copied_bytes(),
    ]);
    counters
}

pub(crate) fn stepped(source: &str, quantum: u64) -> ExpressionEvaluation {
    let compiled = admit(source).expect("corpus sources admit").compile();
    let mut continuation = compiled
        .start(
            &inputs(),
            &worth_foundational::expression_api::ExpressionProfile::interactive(),
        )
        .expect("fixture inputs bind");
    loop {
        match continuation.step(quantum) {
            worth_foundational::expression_api::ExpressionStep::Complete(evaluation) => {
                return evaluation
            }
            worth_foundational::expression_api::ExpressionStep::Suspended(next) => {
                let used = next
                    .cost()
                    .used(worth_foundational::expression_api::ExpressionResource::SemanticWork);
                assert!(used <= 1_000_000, "{source}: suspension keeps the meter");
                continuation = next;
            }
        }
    }
}

pub(crate) fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    worth_foundational::expression_api::ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("fixture limits narrow")
}

pub(crate) fn exceeded(evaluation: &ExpressionEvaluation) -> (ExpressionResource, u64) {
    match evaluation.result().map_err(|denial| denial.detail()) {
        Err(worth_foundational::expression_api::ExpressionDenialDetail::ResourceExceeded {
            resource,
            limit,
        }) => (*resource, *limit),
        other => panic!("expected a resource denial, got {other:?}"),
    }
}
