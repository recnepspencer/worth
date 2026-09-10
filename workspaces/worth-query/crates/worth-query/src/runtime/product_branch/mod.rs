//! Ordinary product-world orchestration exposed by the canonical Query facade.

mod creation;
mod operation_ports;
mod selection;

pub use creation::*;
pub use operation_ports::*;
pub use selection::*;
