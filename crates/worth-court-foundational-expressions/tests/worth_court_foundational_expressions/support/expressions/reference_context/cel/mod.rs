#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "textproto/mod.rs"]
pub(crate) mod textproto;
