mod attempt_binding;

#[path = "declarations.rs"]
mod declarations;
pub use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "copy_intent/mod.rs"]
pub mod copy_intent;
#[allow(dead_code, unused_imports)]
#[path = "target_claim/mod.rs"]
pub mod target_claim;
