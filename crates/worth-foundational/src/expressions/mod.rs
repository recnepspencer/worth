//! The shared typed expression language: bounded parsing, pure admission into
//! canonical typed meaning, installed expression functions, and pure budgeted
//! evaluation over sealed input snapshots.
//!
//! Evaluation is byte-identical only where binary64 arithmetic rounds once per
//! operation with no extended intermediates: x86-64 (SSE2), AArch64, and
//! wasm32, whatever the operating system, so Linux, Windows, and macOS on
//! Intel or Apple silicon are all covered. Other targets are refused at build
//! time rather than silently weakening determinism.

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "wasm32"
)))]
compile_error!("Worth expressions support only x86-64, AArch64, and wasm32 targets");

mod admission;
mod admitted;
mod builder;
mod canonical;
mod compatibility;
mod denial;
mod draft;
mod evaluation;
mod functions;
mod language;
mod numeric;
mod operators;
mod profile;
mod program;
mod syntax;
mod types;

pub use admitted::AdmittedExpression;
pub use builder::{
    Boolean, Dynamic, ExpressionBuilder, ExpressionTypeArgument, Numeric, Ordered, Term, TermKind,
    Textual,
};
pub use canonical::ExpressionProgramIdentity;
pub use denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionDenialFamily, ExpressionOccurrence,
    ExpressionResource, SyntaxDenial,
};
pub use draft::ExpressionDraft;
pub use evaluation::{
    CompiledExpression, ExpressionConsumption, ExpressionContinuation, ExpressionCost,
    ExpressionEvaluation, ExpressionInputs, ExpressionInputsBuilder, ExpressionPathStep,
    ExpressionRead, ExpressionReadKind, ExpressionStep, ExpressionValue, MAX_SLICE_QUANTUM,
};
pub use functions::{
    ExpressionFunctionCatalog, ExpressionFunctionCatalogBuilder, ExpressionFunctionDeclaration,
    InstalledExpressionFunction,
};
pub use language::{expressions, ExpressionLanguage};
pub use profile::ExpressionProfile;
pub use syntax::{ExpressionSourceMap, SourceOrigin, SourceSpan};
pub use types::{
    BaseDimension, ExpressionDimension, ExpressionEnumDeclaration, ExpressionRecordDeclaration,
    ExpressionSchema, ExpressionSchemaBuilder, ExpressionType, ExpressionTypeName, IntegerType,
};

use crate::responsibilities::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "expressions",
        "the shared typed expression language: bounded source parsing, closed V1 types with dimensions, units, and digital-logic buses, pure admission into canonical post-order programs, installed expression functions with exact signatures and acyclic closures, canonical program identity over the canonical-basis digest APIs, and pure evaluation of admitted programs over sealed, type-checked input snapshots in resumable slices with semantic cost accounting and consumed-read recording",
        "binding operands to live owner state, owner authority or currentness, caches, or registries",
    )
}
