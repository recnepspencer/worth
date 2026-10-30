pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::dx::compile::compile_fail_bundle()
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::dx::compile::compile_pass_bundle()
}

pub(crate) use crate::compile_fail::harness::bundle::CompileFailBundle;
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

pub(crate) use crate::shared_support::dx::docs::DocumentationDefaultPathAudit;

pub(crate) fn proof_shape_digest() -> ProofShapeDigest {
    crate::shared_support::dx::digests::proof_shape_digest()
}

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::dx::digests::transition_digest()
}

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::dx::digests::failure_digest()
}

pub(crate) fn codegen_honesty_report() -> CodegenHonestyReport {
    crate::shared_support::dx::codegen::codegen_honesty_report()
}

pub(crate) fn documentation_default_path_audit() -> DocumentationDefaultPathAudit {
    crate::shared_support::dx::docs::documentation_default_path_audit()
}

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::dx::closeout::residual_debt_report()
}
