// Bounded suspension and nested budgets: where slices end never changes an
// evaluation's outcome, reads, or logical cost, and no slice, call, or
// supplied profile resets or widens the admitted limits.
#[allow(unused_imports)]
use crate::expressions::ordinary_context::stepping::*;
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

#[test]
fn a_zero_quantum_still_makes_progress() {
    let evaluation = stepped("[1, 2, 3].map(x, x * 2)", 0);
    assert!(evaluation.result().is_ok());
}

#[test]
fn supplied_profiles_narrow_but_never_widen() {
    let source = "members.map(m, m.thickness * 2.0)";
    let compiled = admit(source).expect("admits").compile();
    let tight = narrowed(
        worth_foundational::expression_api::ExpressionResource::SemanticWork,
        12,
    );
    let evaluation = compiled.evaluate(&inputs(), &tight);
    assert_eq!(
        exceeded(&evaluation),
        (
            worth_foundational::expression_api::ExpressionResource::SemanticWork,
            12
        )
    );
    assert!(
        evaluation
            .cost()
            .used(worth_foundational::expression_api::ExpressionResource::SemanticWork)
            <= 12
    );

    let schema = schema();
    let catalog =
        worth_foundational::expression_api::ExpressionFunctionCatalog::builder(&schema, tight)
            .build();
    let admitted_tight = expressions()
        .parse(source)
        .and_then(|draft| draft.admit(&schema, &catalog, tight))
        .expect("admission does not run the loop");
    let widened = admitted_tight.compile().evaluate(
        &inputs(),
        &worth_foundational::expression_api::ExpressionProfile::engineering(),
    );
    assert_eq!(
        exceeded(&widened),
        (
            worth_foundational::expression_api::ExpressionResource::SemanticWork,
            12
        )
    );

    let visited = narrowed(
        worth_foundational::expression_api::ExpressionResource::VisitedElements,
        2,
    );
    let evaluation = compiled.evaluate(&inputs(), &visited);
    assert_eq!(
        exceeded(&evaluation),
        (
            worth_foundational::expression_api::ExpressionResource::VisitedElements,
            2
        )
    );
}

#[test]
fn installed_calls_share_one_budget() {
    let schema = schema();
    let profile = worth_foundational::expression_api::ExpressionProfile::interactive();
    let mut builder =
        worth_foundational::expression_api::ExpressionFunctionCatalog::builder(&schema, profile);
    builder
        .install(function(
            "fixture::spread",
            &[(
                "n",
                worth_foundational::expression_api::ExpressionType::INT64,
            )],
            worth_foundational::expression_api::ExpressionType::list(
                worth_foundational::expression_api::ExpressionType::INT64,
            ),
            "[n, n, n, n].map(x, x * n)",
        ))
        .expect("installs");
    let catalog = builder.build();
    let admitted = expressions()
        .parse("[1, 2, 3, 4].map(n, sum(fixture::spread(n)))")
        .and_then(|draft| draft.admit(&schema, &catalog, profile))
        .expect("admits");
    let compiled = admitted.compile();
    let whole = compiled.evaluate(&inputs(), &profile);
    assert_eq!(whole.cost().calls(), 4);
    let work = whole
        .cost()
        .used(worth_foundational::expression_api::ExpressionResource::SemanticWork);
    let limit = work - 1;
    let short = compiled.evaluate(
        &inputs(),
        &narrowed(
            worth_foundational::expression_api::ExpressionResource::SemanticWork,
            limit,
        ),
    );
    assert_eq!(
        exceeded(&short),
        (
            worth_foundational::expression_api::ExpressionResource::SemanticWork,
            limit
        )
    );
    assert!(
        short.cost().calls() >= 1,
        "the denial happens inside a later call"
    );
}

/// Equal lists cost the same to compare whether they share one allocation or
/// were built separately, so no outcome depends on how a caller built inputs.
#[test]
fn shared_and_separate_values_compare_at_one_cost() {
    let ty = worth_foundational::expression_api::ExpressionType::list(
        worth_foundational::expression_api::ExpressionType::INT64,
    );
    let schema = worth_foundational::expression_api::ExpressionSchema::builder()
        .operand("a", ty.clone())
        .and_then(|builder| builder.operand("b", ty))
        .expect("operands declare")
        .build();
    let list = || {
        worth_foundational::expression_api::ExpressionValue::list(
            (0..1_000)
                .map(worth_foundational::expression_api::ExpressionValue::integer)
                .collect(),
        )
    };
    let bind = |a, b| {
        worth_foundational::expression_api::ExpressionInputs::builder(&schema)
            .bind("a", a)
            .and_then(|inputs| inputs.bind("b", b))
            .expect("lists bind")
            .build()
    };
    let shared = list();
    let bindings = [bind(shared.clone(), shared), bind(list(), list())];
    for source in ["a == b", "contains([a], b)"] {
        let profile = worth_foundational::expression_api::ExpressionProfile::interactive();
        let compiled = expressions()
            .parse(source)
            .and_then(|draft| draft.admit(&schema, &empty_catalog(&schema), profile))
            .expect("admits")
            .compile();
        let [same, separate] = bindings
            .each_ref()
            .map(|inputs| compiled.evaluate(inputs, &profile));
        assert_eq!(same.result(), separate.result(), "{source}");
        assert_eq!(counters(same.cost()), counters(separate.cost()), "{source}");
        let visited = same
            .cost()
            .used(worth_foundational::expression_api::ExpressionResource::VisitedElements);
        assert!(visited >= 1_000, "{source} visits every pair: {visited}");
        let tight = narrowed(
            worth_foundational::expression_api::ExpressionResource::VisitedElements,
            visited / 2,
        );
        let [same, separate] = bindings
            .each_ref()
            .map(|inputs| compiled.evaluate(inputs, &tight));
        assert_eq!(
            exceeded(&same),
            (
                worth_foundational::expression_api::ExpressionResource::VisitedElements,
                visited / 2
            )
        );
        assert_eq!(same.result(), separate.result(), "{source}");
        assert_eq!(counters(same.cost()), counters(separate.cost()), "{source}");
    }
}

/// Folds charge one unit per iteration, the first element included, on top
/// of their node and one unit per Int64 comparison.
#[test]
fn folds_charge_every_iteration() {
    let work = |source: &str| {
        evaluate(source)
            .cost()
            .used(worth_foundational::expression_api::ExpressionResource::SemanticWork)
    };
    for n in 1..=4_u64 {
        let items: Vec<String> = (1..=n).map(|item| item.to_string()).collect();
        let list = format!("[{}]", items.join(", "));
        let base = work(&list);
        assert_eq!(work(&format!("sum({list})")) - base, 1 + n, "sum of {n}");
        for fold in ["min", "max"] {
            assert_eq!(
                work(&format!("{fold}({list})")) - base,
                1 + n + (n - 1),
                "{fold} of {n}"
            );
        }
        let probe = work(&format!("contains({list}, {n})")) - base;
        assert_eq!(probe, 2 + 2 * n, "contains of {n}: node, probe, n steps");
    }
}
