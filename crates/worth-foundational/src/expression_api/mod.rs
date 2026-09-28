//! Public front door for the shared typed expression language.
//!
//! The common path reads a draft, then admits it against operand types,
//! installed functions, and one resource profile:
//!
//! ```
//! use worth_foundational::expression_api::{
//!     expressions, ExpressionFunctionCatalog, ExpressionProfile, ExpressionSchema, ExpressionType,
//! };
//!
//! let schema = ExpressionSchema::builder()
//!     .operand("width", ExpressionType::Float64)?
//!     .build();
//! let catalog = ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
//! let admitted = expressions()
//!     .parse("width * 2.0")?
//!     .admit(&schema, &catalog, ExpressionProfile::interactive())?;
//! assert_eq!(admitted.result_type(), &ExpressionType::Float64);
//! # Ok::<(), worth_foundational::expression_api::ExpressionDenial>(())
//! ```

mod admission;
mod authoring;

pub use admission::*;
pub use authoring::*;
