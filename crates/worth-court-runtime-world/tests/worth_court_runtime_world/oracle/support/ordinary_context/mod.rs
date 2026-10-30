#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "product/mod.rs"]
pub(crate) mod product;
