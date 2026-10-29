#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[cfg(all())]
#[allow(dead_code, unused_imports)]
#[path = "world/mod.rs"]
pub(crate) mod world;
