mod closeout;
mod codegen;
mod compile;
mod digests;
mod docs;
mod representatives;

pub fn compile_fail_bundle() -> CompileFailBundle {
    compile::compile_fail_bundle()
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    compile::compile_pass_bundle()
}

use crate::shared_support::compile_fail::CompileFailBundle;
use crate::shared_support::compile_pass::CompilePassBundle;
use crate::shared_support::proof_shapes::{FailureDigest, ProofShapeDigest, TransitionDigest};
use crate::shared_support::type_shapes::{CodegenHonestyReport, ResidualDebtReport};

pub use docs::DocumentationDefaultPathAudit;

pub fn proof_shape_digest() -> ProofShapeDigest {
    digests::proof_shape_digest()
}

pub fn transition_digest() -> TransitionDigest {
    digests::transition_digest()
}

pub fn failure_digest() -> FailureDigest {
    digests::failure_digest()
}

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    codegen::codegen_honesty_report()
}

pub fn documentation_default_path_audit() -> DocumentationDefaultPathAudit {
    docs::documentation_default_path_audit()
}

pub fn residual_debt_report() -> ResidualDebtReport {
    closeout::residual_debt_report()
}
