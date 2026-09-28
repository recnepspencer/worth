//! Bounded suspension and nested budgets: where slices end never changes an
//! evaluation's outcome, reads, or logical cost, and no slice, call, or
//! supplied profile resets or widens the admitted limits.

use worth_foundational::expression_api::{
    expressions, ExpressionCost, ExpressionDenialDetail, ExpressionEvaluation,
    ExpressionFunctionCatalog, ExpressionInputs, ExpressionProfile, ExpressionResource,
    ExpressionSchema, ExpressionStep, ExpressionType, ExpressionValue, MAX_SLICE_QUANTUM,
};

use super::{admit, empty_catalog, evaluate, function, inputs, schema};

/// Sources that exercise every resumable step: comparisons, folds, sorts,
/// lookups, text scans, copies, comprehensions, calls, and denials.
const CORPUS: [&str; 16] = [
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

const QUANTA: [u64; 6] = [1, 2, 3, 7, 64, MAX_SLICE_QUANTUM];

const LOGICAL: [ExpressionResource; 6] = [
    ExpressionResource::SemanticWork,
    ExpressionResource::VisitedElements,
    ExpressionResource::InputBytes,
    ExpressionResource::OutputBytes,
    ExpressionResource::ScratchBytes,
    ExpressionResource::RetainedConsumptionBytes,
];

/// Every counter except the slice count, which depends on the quantum.
fn counters(cost: &ExpressionCost) -> Vec<u64> {
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

fn stepped(source: &str, quantum: u64) -> ExpressionEvaluation {
    let compiled = admit(source).expect("corpus sources admit").compile();
    let mut continuation = compiled
        .start(&inputs(), &ExpressionProfile::interactive())
        .expect("fixture inputs bind");
    loop {
        match continuation.step(quantum) {
            ExpressionStep::Complete(evaluation) => return evaluation,
            ExpressionStep::Suspended(next) => {
                let used = next.cost().used(ExpressionResource::SemanticWork);
                assert!(used <= 1_000_000, "{source}: suspension keeps the meter");
                continuation = next;
            }
        }
    }
}

#[test]
fn slice_boundaries_never_change_outcomes_reads_or_cost() {
    for source in CORPUS {
        let whole = evaluate(source);
        assert!(counters(whole.cost())[0] > 0, "{source} did work");
        for quantum in QUANTA {
            let sliced = stepped(source, quantum);
            assert_eq!(sliced.result(), whole.result(), "{source} at {quantum}");
            assert_eq!(
                sliced.consumption(),
                whole.consumption(),
                "{source} at {quantum}"
            );
            assert_eq!(
                counters(sliced.cost()),
                counters(whole.cost()),
                "{source} at {quantum}"
            );
            if quantum == 1 {
                assert!(
                    sliced.cost().slices() > 1,
                    "{source} suspended at quantum 1"
                );
            }
        }
    }
}

#[test]
fn a_zero_quantum_still_makes_progress() {
    let evaluation = stepped("[1, 2, 3].map(x, x * 2)", 0);
    assert!(evaluation.result().is_ok());
}

fn narrowed(resource: ExpressionResource, limit: u64) -> ExpressionProfile {
    ExpressionProfile::interactive()
        .narrowed(resource, limit)
        .expect("fixture limits narrow")
}

fn exceeded(evaluation: &ExpressionEvaluation) -> (ExpressionResource, u64) {
    match evaluation.result().map_err(|denial| denial.detail()) {
        Err(ExpressionDenialDetail::ResourceExceeded { resource, limit }) => (*resource, *limit),
        other => panic!("expected a resource denial, got {other:?}"),
    }
}

#[test]
fn supplied_profiles_narrow_but_never_widen() {
    let source = "members.map(m, m.thickness * 2.0)";
    let compiled = admit(source).expect("admits").compile();
    let tight = narrowed(ExpressionResource::SemanticWork, 12);
    let evaluation = compiled.evaluate(&inputs(), &tight);
    assert_eq!(
        exceeded(&evaluation),
        (ExpressionResource::SemanticWork, 12)
    );
    assert!(evaluation.cost().used(ExpressionResource::SemanticWork) <= 12);

    let schema = schema();
    let catalog = ExpressionFunctionCatalog::builder(&schema, tight).build();
    let admitted_tight = expressions()
        .parse(source)
        .and_then(|draft| draft.admit(&schema, &catalog, tight))
        .expect("admission does not run the loop");
    let widened = admitted_tight
        .compile()
        .evaluate(&inputs(), &ExpressionProfile::engineering());
    assert_eq!(exceeded(&widened), (ExpressionResource::SemanticWork, 12));

    let visited = narrowed(ExpressionResource::VisitedElements, 2);
    let evaluation = compiled.evaluate(&inputs(), &visited);
    assert_eq!(
        exceeded(&evaluation),
        (ExpressionResource::VisitedElements, 2)
    );
}

#[test]
fn installed_calls_share_one_budget() {
    let schema = schema();
    let profile = ExpressionProfile::interactive();
    let mut builder = ExpressionFunctionCatalog::builder(&schema, profile);
    builder
        .install(function(
            "fixture::spread",
            &[("n", ExpressionType::INT64)],
            ExpressionType::list(ExpressionType::INT64),
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
    let work = whole.cost().used(ExpressionResource::SemanticWork);
    let limit = work - 1;
    let short = compiled.evaluate(
        &inputs(),
        &narrowed(ExpressionResource::SemanticWork, limit),
    );
    assert_eq!(exceeded(&short), (ExpressionResource::SemanticWork, limit));
    assert!(
        short.cost().calls() >= 1,
        "the denial happens inside a later call"
    );
}

#[test]
fn exhaustion_is_identical_under_every_quantum() {
    let source = r#"{"delta": 4, "alpha": 1, "charlie": 3, "bravo": 2, "echo": 5}"#;
    let compiled = admit(source).expect("admits").compile();
    let whole = compiled.evaluate(&inputs(), &ExpressionProfile::interactive());
    let work = whole.cost().used(ExpressionResource::SemanticWork);
    for limit in [1, work / 3, work / 2, work - 1] {
        let profile = narrowed(ExpressionResource::SemanticWork, limit);
        let expected = compiled.evaluate(&inputs(), &profile);
        for quantum in QUANTA {
            let mut continuation = compiled.start(&inputs(), &profile).expect("binds");
            let sliced = loop {
                match continuation.step(quantum) {
                    ExpressionStep::Complete(evaluation) => break evaluation,
                    ExpressionStep::Suspended(next) => continuation = next,
                }
            };
            assert_eq!(
                sliced.result(),
                expected.result(),
                "limit {limit} at {quantum}"
            );
            assert_eq!(counters(sliced.cost()), counters(expected.cost()));
        }
    }
}

#[test]
fn byte_limits_deny_at_the_same_safe_point_under_every_quantum() {
    let built = r#"[1, 2, 3, 4, 5, 6, 7, 8].map(x, "item")"#;
    let read = "members.filter(m, m.material == Material::Steel).map(m, m.thickness * 2.0)";
    for (source, resource) in [
        (built, ExpressionResource::OutputBytes),
        (built, ExpressionResource::ScratchBytes),
        (read, ExpressionResource::InputBytes),
        (read, ExpressionResource::RetainedConsumptionBytes),
    ] {
        let compiled = admit(source).expect("admits").compile();
        let whole = compiled.evaluate(&inputs(), &ExpressionProfile::interactive());
        let used = whole.cost().used(resource);
        assert!(used > 1, "{resource:?} is charged");
        let profile = narrowed(resource, used / 2);
        let expected = compiled.evaluate(&inputs(), &profile);
        assert_eq!(exceeded(&expected), (resource, used / 2));
        assert!(
            expected.cost().used(resource) <= used / 2,
            "{resource:?} never overdraws"
        );
        for quantum in QUANTA {
            let mut continuation = compiled.start(&inputs(), &profile).expect("binds");
            let sliced = loop {
                match continuation.step(quantum) {
                    ExpressionStep::Complete(evaluation) => break evaluation,
                    ExpressionStep::Suspended(next) => continuation = next,
                }
            };
            assert_eq!(
                sliced.result(),
                expected.result(),
                "{resource:?} at {quantum}"
            );
            assert_eq!(counters(sliced.cost()), counters(expected.cost()));
        }
    }
}

#[test]
fn cancelling_between_slices_leaves_nothing_behind() {
    let source = "members.filter(m, m.material == Material::Steel).map(m, m.thickness * 2.0)";
    let compiled = admit(source).expect("admits").compile();
    let whole = compiled.evaluate(&inputs(), &ExpressionProfile::interactive());
    for slices in 1..8 {
        let mut continuation = compiled
            .start(&inputs(), &ExpressionProfile::interactive())
            .expect("binds");
        for _ in 0..slices {
            continuation = match continuation.step(1) {
                ExpressionStep::Suspended(next) => next,
                ExpressionStep::Complete(_) => panic!("quantum 1 suspends this source"),
            };
        }
        // Cancellation is dropping the continuation: it holds no shared
        // state, so a fresh evaluation is unaffected.
        drop(continuation);
        let again = compiled.evaluate(&inputs(), &ExpressionProfile::interactive());
        assert_eq!(again.result(), whole.result());
        assert_eq!(again.consumption(), whole.consumption());
        assert_eq!(counters(again.cost()), counters(whole.cost()));
    }
}

/// Equal lists cost the same to compare whether they share one allocation or
/// were built separately, so no outcome depends on how a caller built inputs.
#[test]
fn shared_and_separate_values_compare_at_one_cost() {
    let ty = ExpressionType::list(ExpressionType::INT64);
    let schema = ExpressionSchema::builder()
        .operand("a", ty.clone())
        .and_then(|builder| builder.operand("b", ty))
        .expect("operands declare")
        .build();
    let list = || ExpressionValue::list((0..1_000).map(ExpressionValue::integer).collect());
    let bind = |a, b| {
        ExpressionInputs::builder(&schema)
            .bind("a", a)
            .and_then(|inputs| inputs.bind("b", b))
            .expect("lists bind")
            .build()
    };
    let shared = list();
    let bindings = [bind(shared.clone(), shared), bind(list(), list())];
    for source in ["a == b", "contains([a], b)"] {
        let profile = ExpressionProfile::interactive();
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
        let visited = same.cost().used(ExpressionResource::VisitedElements);
        assert!(visited >= 1_000, "{source} visits every pair: {visited}");
        let tight = narrowed(ExpressionResource::VisitedElements, visited / 2);
        let [same, separate] = bindings
            .each_ref()
            .map(|inputs| compiled.evaluate(inputs, &tight));
        assert_eq!(
            exceeded(&same),
            (ExpressionResource::VisitedElements, visited / 2)
        );
        assert_eq!(same.result(), separate.result(), "{source}");
        assert_eq!(counters(same.cost()), counters(separate.cost()), "{source}");
    }
}
