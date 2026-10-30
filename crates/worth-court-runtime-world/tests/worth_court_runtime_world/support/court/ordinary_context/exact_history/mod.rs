#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "memory/mod.rs"]
pub(crate) mod memory;
