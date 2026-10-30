#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "residency_geometry/mod.rs"]
pub(crate) mod residency_geometry;
