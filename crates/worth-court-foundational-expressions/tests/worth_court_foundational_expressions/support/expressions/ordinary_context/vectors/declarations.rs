// Fixed numeric byte vectors. Each expected bit pattern follows from
// IEEE 754 binary64 with one nearest-even rounding per declared operation,
// so the same vectors pass unchanged on every supported target; a target
// that fused, reassociated, flushed, or widened would fail here.
pub(crate) use worth_foundational::expression_api::ExpressionDenialFamily as Family;

pub(crate) use {
    crate::expressions::ordinary_context::evaluation_denial,
    crate::expressions::ordinary_context::value,
};

pub(crate) fn bits(source: &str) -> u64 {
    match value(source).as_f64() {
        Some(float) => float.to_bits(),
        None => panic!("{source} is not a Float64"),
    }
}
