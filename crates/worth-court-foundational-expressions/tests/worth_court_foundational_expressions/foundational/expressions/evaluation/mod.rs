// Evaluation semantics against fixed expected values. Every expected value
// is written out by hand from the language rules, never read back from the
// evaluator.
#[allow(unused_imports)] use crate::expressions::ordinary_context::evaluation::*;
pub(crate) use {worth_foundational::expression_api::ExpressionDenialDetail, worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionOccurrence, worth_foundational::expression_api::ExpressionValue};

pub(crate) use {crate::expressions::ordinary_context::evaluation_denial, crate::expressions::ordinary_context::float, crate::expressions::ordinary_context::meters, crate::expressions::ordinary_context::value};

#[test]
fn arithmetic_is_checked_and_exact() {
    assert_eq!(value("width * 2.0 - depth"), float(4.5));
    assert_eq!(value("count + 1"), int(8));
    assert_eq!(value("count / 2"), int(3));
    assert_eq!(value("-7 / 2"), int(-3));
    assert_eq!(value("-7 % 3"), int(-1));
    assert_eq!(value("abs(small)"), int(3));
    assert_eq!(value("-small"), int(3));
    assert_eq!(
        family("9223372036854775807 + 1"),
        worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow
    );
    assert_eq!(
        family("exact_cast<Int32>(count * 1000000000)"),
        worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow
    );
    assert_eq!(family("count / 0"), worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero);
    assert_eq!(family("count % 0"), worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero);
    assert_eq!(family("1.0 / 0.0"), worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero);
    assert_eq!(family("sqrt(-1.0)"), worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticDomain);
    assert_eq!(value("sqrt(width * 10.0)"), float(5.0));
}

#[test]
fn decimals_round_only_where_named() {
    assert_eq!(
        value(r#"decimal("1.10") + decimal("2.205")"#),
        decimal(3305, 3)
    );
    assert_eq!(
        value(r#"decimal_div(decimal("1"), decimal("3"), 4, Rounding::NearestEven)"#),
        decimal(3333, 4)
    );
    assert_eq!(
        value(r#"quantize(decimal("2.345"), 2, Rounding::NearestEven)"#),
        decimal(234, 2)
    );
    assert_eq!(
        value(r#"quantize(decimal("-2.345"), 2, Rounding::TowardNegative)"#),
        decimal(-235, 2)
    );
    assert_eq!(
        family(r#"decimal_div(decimal("1"), decimal("0"), 2, Rounding::TowardZero)"#),
        worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero
    );
}

#[test]
fn quantities_convert_at_the_boundary() {
    assert_eq!(value("magnitude(clear_width, mm)"), float(900.0));
    assert_eq!(
        value("quantity(900.0, mm) <= clear_width"),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(value("clear_width - frame.thickness * 2.0"), meters(0.88));
    assert_eq!(
        value("near(clear_width, quantity(0.9000001, m), quantity(1.0, mm), 0.0)"),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
}

#[test]
fn logic_short_circuits_left_to_right() {
    assert_eq!(
        value("ready || count / 0 == 1"),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value("!ready && count / 0 == 1"),
        worth_foundational::expression_api::ExpressionValue::bool(false)
    );
    assert_eq!(family("ready && count / 0 == 1"), worth_foundational::expression_api::ExpressionDenialFamily::DivisionByZero);
    assert_eq!(value("ready ? 1 : count / 0"), int(1));
    assert_eq!(value("let twice = width * 2.0; twice + twice"), float(10.0));
}

#[test]
fn absence_is_explicit() {
    assert_eq!(
        value(r#"label ?? "Nothing selected""#),
        text("Nothing selected")
    );
    assert_eq!(value("is_some(label)"), worth_foundational::expression_api::ExpressionValue::bool(false));
    assert_eq!(value(r#"some("x") ?? "y""#), text("x"));
    assert_eq!(family("unwrap(label)"), worth_foundational::expression_api::ExpressionDenialFamily::AbsentValue);
    assert_eq!(
        evaluation_denial("unwrap(label)").detail(),
        &worth_foundational::expression_api::ExpressionDenialDetail::AbsentValue
    );
}

#[test]
fn comprehensions_keep_order() {
    assert_eq!(
        value("members.filter(m, m.material == Material::Steel).map(m, m.thickness)"),
        worth_foundational::expression_api::ExpressionValue::list(vec![meters(0.01), meters(0.03)])
    );
    assert_eq!(
        value("members.all(m, m.thickness > quantity(1.0, mm))"),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value("members.any(m, m.material == Material::Timber)"),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value("[1, 2, 3].map(x, [1, 2, 3].map(y, x * y))"),
        worth_foundational::expression_api::ExpressionValue::list(vec![ints([1, 2, 3]), ints([2, 4, 6]), ints([3, 6, 9])])
    );
}

#[test]
fn ordering_builtins_keep_the_first_tie() {
    assert_eq!(value("min(3, count)"), int(3));
    assert_eq!(value("max(width, depth)"), float(2.5));
    assert_eq!(value("clamp(count, 0, 5)"), int(5));
    assert_eq!(value("clamp(count - 10, 0, 5)"), int(0));
    assert_eq!(family("clamp(1, 5, 0)"), worth_foundational::expression_api::ExpressionDenialFamily::Bounds);
    assert_eq!(value("min([3, 1, 2])"), some(int(1)));
    assert_eq!(value("max([3, 1, 2])"), some(int(3)));
    assert_eq!(value("sum([1.5, 2.5, 3.0])"), float(7.0));
    assert_eq!(value(r#""abc" < "abd""#), worth_foundational::expression_api::ExpressionValue::bool(true));
    assert_eq!(value("[1, 2] != [1, 2, 0]"), worth_foundational::expression_api::ExpressionValue::bool(true));
}

#[test]
fn text_counts_unicode_scalars() {
    assert_eq!(value(r#"length("héllo")"#), int(5));
    assert_eq!(value(r#"slice("héllo", 1, 3)"#), some(text("él")));
    assert_eq!(value(r#"slice("héllo", 3, 9)"#), worth_foundational::expression_api::ExpressionValue::none());
    assert_eq!(
        value(r#"contains("engineering", "gin")"#),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value(r#"contains("engineering", "gun")"#),
        worth_foundational::expression_api::ExpressionValue::bool(false)
    );
    assert_eq!(
        value(r#"starts_with("engineering", "engine")"#),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value(r#"ends_with("engineering", "ring")"#),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(
        value(r#"ends_with("ring", "engineering")"#),
        worth_foundational::expression_api::ExpressionValue::bool(false)
    );
}

#[test]
fn maps_are_canonical_and_keys_unique() {
    assert_eq!(value(r#"{"b": 2, "a": 1}"#), value(r#"{"a": 1, "b": 2}"#));
    assert_eq!(value(r#"get({"b": 2, "a": 1}, "a")"#), some(int(1)));
    assert_eq!(value(r#"get({"b": 2}, "a")"#), worth_foundational::expression_api::ExpressionValue::none());
    assert_eq!(
        value(r#"contains({"b": 2, "a": 1}, "b")"#),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
    assert_eq!(value(r#"length({"b": 2, "a": 1, "c": 3})"#), int(3));
    assert_eq!(
        value(r#"entries({"b": 2, "a": 1}).map(e, e.key)"#),
        worth_foundational::expression_api::ExpressionValue::list(vec![text("a"), text("b")])
    );
    assert_eq!(family(r#"{"a": 1, "b": 2, "a": 3}"#), worth_foundational::expression_api::ExpressionDenialFamily::InvalidValue);
    assert_eq!(value("get([10, 20], 1)"), some(int(20)));
    assert_eq!(value("get([10, 20], 2)"), worth_foundational::expression_api::ExpressionValue::none());
    assert_eq!(value("get([10, 20], -1)"), worth_foundational::expression_api::ExpressionValue::none());
}

#[test]
fn buses_follow_their_widths() {
    let bits = |width, limb| worth_foundational::expression_api::ExpressionValue::bits(width, vec![limb]).expect("fixture bus");
    assert_eq!(
        value(r#"bit_and(bus, bits<8>("00001111"))"#),
        bits(8, 0b0101)
    );
    assert_eq!(value("slice<2, 6>(bus)"), bits(4, 0b1001));
    assert_eq!(
        value(r#"concat(bits<4>("1100"), bits<4>("0011"))"#),
        bits(8, 0b1100_0011)
    );
    assert_eq!(value("shift_left(bus, 1)"), bits(8, 0b0100_1010));
    assert_eq!(value("exact_cast<UInt8>(bus)"), int(0b1010_0101));
    assert_eq!(
        family(r#"bits_add(bits<4>("1111"), bits<4>("0001"))"#),
        worth_foundational::expression_api::ExpressionDenialFamily::ArithmeticOverflow
    );
    assert_eq!(
        value(r#"wrapping_add(bits<4>("1111"), bits<4>("0001"))"#),
        bits(4, 0)
    );
    assert_eq!(
        value(r#"case_equal(enable, logic4("1"))"#),
        worth_foundational::expression_api::ExpressionValue::bool(true)
    );
}

#[test]
fn denials_carry_their_source_occurrence() {
    let denial = evaluation_denial("count + count / 0");
    let worth_foundational::expression_api::ExpressionOccurrence::Source(span) = denial.occurrence().expect("an occurrence") else {
        panic!("parsed source maps to spans");
    };
    assert_eq!((span.start(), span.end()), (8, 17));
}
