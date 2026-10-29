#[path = "codegen.rs"]
pub(crate) mod codegen;

#[path = "debt.rs"]
pub(crate) mod debt;

#[path = "type_shape.rs"]
pub(crate) mod type_shape;

pub use codegen::{CodegenHonestyReport, CodegenShapeCheck};
pub use debt::{DebtInventory, DebtItem, ResidualDebtReport};
pub use type_shape::{TypeShapeCheck, TypeShapeReport};
