#[allow(unused_imports)]
pub(crate) use {crate::expressions::ordinary_context::stepping::CORPUS, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionResource, worth_foundational::expression_api::ExpressionStep, crate::expressions::ordinary_context::stepping::QUANTA, crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::stepping::counters, crate::expressions::ordinary_context::evaluate, crate::expressions::ordinary_context::stepping::exceeded, crate::expressions::ordinary_context::inputs, crate::expressions::ordinary_context::stepping::narrowed, crate::expressions::ordinary_context::stepping::stepped};

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
fn exhaustion_is_identical_under_every_quantum() {
    let source = r#"{"delta": 4, "alpha": 1, "charlie": 3, "bravo": 2, "echo": 5}"#;
    let compiled = admit(source).expect("admits").compile();
    let whole = compiled.evaluate(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive());
    let work = whole.cost().used(worth_foundational::expression_api::ExpressionResource::SemanticWork);
    for limit in [1, work / 3, work / 2, work - 1] {
        let profile = narrowed(worth_foundational::expression_api::ExpressionResource::SemanticWork, limit);
        let expected = compiled.evaluate(&inputs(), &profile);
        for quantum in QUANTA {
            let mut continuation = compiled.start(&inputs(), &profile).expect("binds");
            let sliced = loop {
                match continuation.step(quantum) {
                    worth_foundational::expression_api::ExpressionStep::Complete(evaluation) => break evaluation,
                    worth_foundational::expression_api::ExpressionStep::Suspended(next) => continuation = next,
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
        (built, worth_foundational::expression_api::ExpressionResource::OutputBytes),
        (built, worth_foundational::expression_api::ExpressionResource::ScratchBytes),
        (read, worth_foundational::expression_api::ExpressionResource::InputBytes),
        (read, worth_foundational::expression_api::ExpressionResource::RetainedConsumptionBytes),
    ] {
        let compiled = admit(source).expect("admits").compile();
        let whole = compiled.evaluate(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive());
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
                    worth_foundational::expression_api::ExpressionStep::Complete(evaluation) => break evaluation,
                    worth_foundational::expression_api::ExpressionStep::Suspended(next) => continuation = next,
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
    let whole = compiled.evaluate(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive());
    for slices in 1..8 {
        let mut continuation = compiled
            .start(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive())
            .expect("binds");
        for _ in 0..slices {
            continuation = match continuation.step(1) {
                worth_foundational::expression_api::ExpressionStep::Suspended(next) => next,
                worth_foundational::expression_api::ExpressionStep::Complete(_) => panic!("quantum 1 suspends this source"),
            };
        }
        // Cancellation is dropping the continuation: it holds no shared
        // state, so a fresh evaluation is unaffected.
        drop(continuation);
        let again = compiled.evaluate(&inputs(), &worth_foundational::expression_api::ExpressionProfile::interactive());
        assert_eq!(again.result(), whole.result());
        assert_eq!(again.consumption(), whole.consumption());
        assert_eq!(counters(again.cost()), counters(whole.cost()));
    }
}
