mod durability_policy;
pub use self::durability_policy::serving_durability;

#[path = "declarations.rs"]
mod declarations;
pub use self::declarations::*;
#[cfg(all())]
#[allow(dead_code, unused_imports)]
#[path = "empty_no_release/mod.rs"]
pub mod empty_no_release;
#[cfg(all())]
#[allow(dead_code, unused_imports)]
#[path = "plain_serving/mod.rs"]
pub mod plain_serving;
#[cfg(all())]
#[allow(dead_code, unused_imports)]
#[path = "recovery/mod.rs"]
pub mod recovery;
#[cfg(all())]
#[allow(dead_code, unused_imports)]
#[path = "recovery_request/mod.rs"]
pub mod recovery_request;
