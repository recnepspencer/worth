#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "extent/mod.rs"]
pub(crate) mod extent;
#[allow(dead_code, unused_imports)]
#[path = "inline/mod.rs"]
pub(crate) mod inline;
