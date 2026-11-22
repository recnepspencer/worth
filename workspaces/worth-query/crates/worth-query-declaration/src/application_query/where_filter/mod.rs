//! Parameterized residual row filtering for ordinary application queries.
//!
//! A where filter is one Boolean Foundation expression over named operands:
//! declared fields of the query's root entity and the query's typed
//! parameters. Foundation owns syntax, typing, identity and metered
//! evaluation; this module only binds operand slots and admits the program
//! statically. Index-seeking selection (`where_equal`, root paths) stays
//! structured; the filter narrows the selected candidate rows afterwards.
mod authoring;
mod denial;
mod operand;
mod program;
pub(super) mod validation;

pub use authoring::{ApplicationQueryWhereFilter, ApplicationQueryWhereFilterOperands};
pub use denial::ApplicationQueryWhereFilterDenial;
pub use operand::{ApplicationQueryWhereFilterOperand, ApplicationQueryWhereFilterSource};
pub use program::ApplicationQueryWhereFilterProgram;

#[cfg(test)]
mod tests;
