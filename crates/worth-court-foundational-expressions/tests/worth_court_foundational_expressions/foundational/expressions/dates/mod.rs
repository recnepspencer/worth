// Native dates: lossless correspondence with the foundational carrier and
// total ordering by calendar day, under any slice quantum.
use proptest::prelude::*;
use worth_foundational::expression_api::{
    expressions, AdmittedExpression, ExpressionFunctionCatalog, ExpressionInputs,
    ExpressionProfile, ExpressionSchema, ExpressionStep, ExpressionType, ExpressionValue,
};
use worth_foundational::facade::{AspectValue, CanonicalDate, ScalarAspectType};

fn schema() -> ExpressionSchema {
    ExpressionSchema::builder()
        .operand("due", ExpressionType::Date)
        .and_then(|builder| builder.operand("start", ExpressionType::Date))
        .and_then(|builder| builder.operand("closed", ExpressionType::option(ExpressionType::Date)))
        .expect("fixture operands are valid")
        .build()
}

fn admit(source: &str) -> AdmittedExpression {
    let schema = schema();
    let catalog =
        ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    expressions()
        .parse(source)
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .unwrap_or_else(|denial| panic!("{source}: {denial:?}"))
}

fn day(days_from_unix_epoch: i32) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch,
    }
}

fn inputs(due: i32, start: i32, closed: Option<i32>) -> ExpressionInputs {
    let closed = match closed {
        Some(days) => ExpressionValue::some(ExpressionValue::date(day(days))),
        None => ExpressionValue::none(),
    };
    ExpressionInputs::builder(&schema())
        .bind("due", ExpressionValue::date(day(due)))
        .and_then(|builder| builder.bind("start", ExpressionValue::date(day(start))))
        .and_then(|builder| builder.bind("closed", closed))
        .expect("fixture inputs conform")
        .build()
}

/// The value and semantic work of a run sliced at `quantum`.
fn stepped(
    admitted: &AdmittedExpression,
    inputs: &ExpressionInputs,
    quantum: u64,
) -> (ExpressionValue, u64) {
    let mut continuation = admitted
        .compile()
        .start(inputs, &ExpressionProfile::interactive())
        .expect("inputs bind");
    loop {
        match continuation.step(quantum) {
            ExpressionStep::Complete(evaluation) => {
                let work = evaluation
                    .cost()
                    .used(worth_foundational::expression_api::ExpressionResource::SemanticWork);
                return (evaluation.into_result().expect("evaluates"), work);
            }
            ExpressionStep::Suspended(next) => continuation = next,
        }
    }
}

#[test]
fn native_dates_correspond_losslessly_with_the_date_type() {
    assert_eq!(
        ExpressionType::from_scalar_family(ScalarAspectType::Date).unwrap(),
        ExpressionType::Date
    );
    for days in [i32::MIN, -1, 0, 20_413, i32::MAX] {
        let value = ExpressionValue::from_aspect_value(&AspectValue::Date(day(days))).unwrap();
        assert_eq!(value.as_date(), Some(day(days)));
    }
    let admitted = admit("min(due, start)");
    assert_eq!(admitted.result_type(), &ExpressionType::Date);
    let earliest = admitted
        .compile()
        .evaluate(&inputs(9, -3, None), &ExpressionProfile::interactive());
    assert_eq!(earliest.into_result().unwrap().as_date(), Some(day(-3)));
}

#[test]
fn dates_have_no_arithmetic_and_compare_only_with_dates() {
    let schema = schema();
    let catalog =
        ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    for source in ["due + start", "due < 3", "-due", "contains(due, start)"] {
        let admitted = expressions()
            .parse(source)
            .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()));
        assert!(admitted.is_err(), "{source} must be denied");
    }
}

const COMPARISONS: [&str; 6] = [
    "due == start",
    "due != start",
    "due < start",
    "due <= start",
    "due > start",
    "due >= start",
];

proptest! {
    #[test]
    fn date_comparisons_follow_calendar_order_under_any_quantum(
        due in any::<i32>(),
        start in any::<i32>(),
        quantum in 0_u64..4,
    ) {
        let expected = [due == start, due != start, due < start, due <= start, due > start, due >= start];
        let bound = inputs(due, start, None);
        for (source, expected) in COMPARISONS.iter().zip(expected) {
            let admitted = admit(source);
            let (whole, work) = stepped(&admitted, &bound, u64::MAX);
            let (sliced, sliced_work) = stepped(&admitted, &bound, quantum);
            prop_assert_eq!(whole.as_bool(), Some(expected), "{}", source);
            prop_assert_eq!(sliced, whole);
            prop_assert_eq!(sliced_work, work);
        }
    }

    #[test]
    fn optional_dates_compare_through_explicit_fallbacks(
        due in any::<i32>(),
        start in any::<i32>(),
        closed in proptest::option::of(any::<i32>()),
    ) {
        let admitted = admit("(closed ?? start) < due && (closed == none<Date>) == !is_some(closed)");
        let (value, _) = stepped(&admitted, &inputs(due, start, closed), u64::MAX);
        prop_assert_eq!(value.as_bool(), Some(closed.unwrap_or(start) < due));
    }
}

#[test]
fn date_program_identity_is_deterministic_across_spelling_and_decoding() {
    let reference = admit("due <= start");
    let respelled = admit("(due)   <=   (start)");
    assert_eq!(reference.identity(), respelled.identity());
    let encoded = expressions().parse("due <= start").unwrap().encode();
    let schema = schema();
    let catalog =
        ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
    let decoded = expressions()
        .decode(&encoded)
        .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
        .expect("decoded Date drafts admit");
    assert_eq!(decoded.identity(), reference.identity());
    let strict = admit("due < start");
    assert_ne!(strict.identity(), reference.identity());
    assert_ne!(strict.identity().digest(), reference.identity().digest());
}
