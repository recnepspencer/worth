// Consumed reads: an evaluation records exactly the operand paths its
// outcome depended on, at the weakest kind that decided it, and binding
// checks every value before evaluation can read it.
pub(crate) use {worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionInputs, worth_foundational::expression_api::ExpressionPathStep as Step, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionReadKind as Kind, worth_foundational::expression_api::ExpressionValue};

pub(crate) use {crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::evaluate, crate::expressions::ordinary_context::frame, crate::expressions::ordinary_context::schema};

/// `(operand, path, kind)` for every recorded read, in order.
pub(crate) fn reads(source: &str) -> Vec<(String, Vec<Step>, Kind)> {
    evaluate(source)
        .consumption()
        .reads()
        .iter()
        .map(|read| {
            (
                read.operand().to_string(),
                read.path().to_vec(),
                read.kind(),
            )
        })
        .collect()
}

pub(crate) fn read(operand: &str, path: &[Step], kind: Kind) -> (String, Vec<Step>, Kind) {
    (operand.to_string(), path.to_vec(), kind)
}