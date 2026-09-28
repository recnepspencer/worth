//! Evaluation: compiled plans, sealed inputs, stepped execution, consumed
//! reads, and cost.

pub use crate::expressions::{
    CompiledExpression, ExpressionConsumption, ExpressionContinuation, ExpressionCost,
    ExpressionEvaluation, ExpressionInputs, ExpressionInputsBuilder, ExpressionPathStep,
    ExpressionRead, ExpressionReadKind, ExpressionStep, ExpressionValue, MAX_SLICE_QUANTUM,
};
