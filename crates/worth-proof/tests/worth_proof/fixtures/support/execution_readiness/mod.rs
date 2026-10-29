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
use super::proof_shapes::{BasisDigest, FailureDigest, TransitionDigest};
use super::type_shapes::{CodegenHonestyReport, ResidualDebtReport};

pub fn compile_fail_bundle() -> CompileFailBundle {
    compile::compile_fail_bundle()
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    compile::compile_pass_bundle()
}

pub fn basis_digest() -> BasisDigest {
    digests::basis_digest()
}

pub fn failure_digest() -> FailureDigest {
    digests::failure_digest()
}

pub fn transition_digest() -> TransitionDigest {
    digests::transition_digest()
}

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    codegen::codegen_honesty_report()
}

pub fn residual_debt_report() -> ResidualDebtReport {
    closeout::residual_debt_report()
}
