pub(crate) use crate::shared_support::compile_fail::bundle::CompileFailBundle;
pub(crate) use crate::shared_support::compile_pass::bundle::CompilePassBundle;
pub(crate) use {
    crate::shared_support::proof_shapes::digest::FailureDigest,
    crate::shared_support::proof_shapes::digest::ProofShapeDigest,
    crate::shared_support::proof_shapes::digest::TransitionDigest,
};
pub(crate) use {
    crate::shared_support::type_shapes::codegen::CodegenHonestyReport,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::composition_family::compile::compile_fail_bundle()
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::composition_family::compile::compile_pass_bundle()
}

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::composition_family::digests::transition_digest()
}

pub(crate) fn composition_digest() -> ProofShapeDigest {
    crate::shared_support::composition_family::digests::composition_digest()
}

pub(crate) fn proof_shape_digest() -> ProofShapeDigest {
    crate::shared_support::composition_family::digests::proof_shape_digest()
}

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::composition_family::digests::failure_digest()
}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::composition_family::codegen::codegen_honesty_report()
}

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::composition_family::closeout::residual_debt_report()
}
