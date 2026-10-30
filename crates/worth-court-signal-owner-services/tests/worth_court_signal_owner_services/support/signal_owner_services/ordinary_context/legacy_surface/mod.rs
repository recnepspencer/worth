#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "sealed_inspection/mod.rs"]
pub(crate) mod sealed_inspection;
