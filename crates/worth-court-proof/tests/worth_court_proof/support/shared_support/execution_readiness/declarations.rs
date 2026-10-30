pub(crate) use crate::compile_fail::harness::bundle::CompileFailBundle;
pub(crate) use crate::shared_support::compile_pass::bundle::CompilePassBundle;
pub(crate) use {
    crate::shared_support::proof_shapes::digest::BasisDigest,
    crate::shared_support::proof_shapes::digest::FailureDigest,
    crate::shared_support::proof_shapes::digest::TransitionDigest,
};
pub(crate) use {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::execution_readiness::compile::compile_fail_bundle()
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::execution_readiness::compile::compile_pass_bundle()
}

pub(crate) fn basis_digest() -> BasisDigest {
    crate::shared_support::execution_readiness::digests::basis_digest()
}

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::execution_readiness::digests::failure_digest()
}

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::execution_readiness::digests::transition_digest()
}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::execution_readiness::codegen::codegen_honesty_report()
}

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::execution_readiness::closeout::residual_debt_report()
}
