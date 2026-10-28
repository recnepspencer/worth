#[path = "closeout.rs"]
pub(crate) mod closeout;

#[path = "codegen.rs"]
pub(crate) mod codegen;

#[path = "compile.rs"]
pub(crate) mod compile;

#[path = "digests.rs"]
pub(crate) mod digests;

#[path = "representatives.rs"]
pub(crate) mod representatives;

use super::compile_fail::CompileFailBundle;
use super::compile_pass::CompilePassBundle;
use super::proof_shapes::{FailureDigest, ProofShapeDigest, TransitionDigest};
use super::type_shapes::{CodegenHonestyReport, ResidualDebtReport};

pub fn compile_fail_bundle() -> CompileFailBundle {
    compile::compile_fail_bundle()
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    compile::compile_pass_bundle()
}

pub fn transition_digest() -> TransitionDigest {
    digests::transition_digest()
}

pub fn composition_digest() -> ProofShapeDigest {
    digests::composition_digest()
}

pub fn proof_shape_digest() -> ProofShapeDigest {
    digests::proof_shape_digest()
}

pub fn failure_digest() -> FailureDigest {
    digests::failure_digest()
}

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    codegen::codegen_honesty_report()
}

pub fn residual_debt_report() -> ResidualDebtReport {
    closeout::residual_debt_report()
}
