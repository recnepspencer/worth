#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "combined_cancellation/mod.rs"]
pub(crate) mod combined_cancellation;
#[allow(dead_code, unused_imports)]
#[path = "combined_race/mod.rs"]
pub(crate) mod combined_race;
