pub(crate) use crate::shared_support::compile_fail::bundle::CompileFailBundle;
pub(crate) use crate::shared_support::compile_pass::bundle::CompilePassBundle;
pub(crate) use {
    crate::shared_support::proof_shapes::digest::FailureDigest,
    crate::shared_support::proof_shapes::digest::TransitionDigest,
};
pub(crate) use {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::transition_algebra::compile::compile_fail_bundle()
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::transition_algebra::compile::compile_pass_bundle()
}

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::transition_algebra::digests::transition_digest()
}

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::transition_algebra::digests::failure_digest()
}

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::transition_algebra::closeout::residual_debt_report()
}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::transition_algebra::codegen::codegen_honesty_report()
}
