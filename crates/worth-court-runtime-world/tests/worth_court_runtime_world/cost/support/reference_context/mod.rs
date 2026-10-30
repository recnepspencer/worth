#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "population/mod.rs"]
pub(crate) mod population;
#[allow(dead_code, unused_imports)]
#[path = "writers/mod.rs"]
pub(crate) mod writers;
