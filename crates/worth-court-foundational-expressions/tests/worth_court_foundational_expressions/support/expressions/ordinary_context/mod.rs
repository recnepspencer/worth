#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;
#[allow(dead_code, unused_imports)]
#[path = "canonical/mod.rs"]
pub(crate) mod canonical;
#[allow(dead_code, unused_imports)]
#[path = "conformance/mod.rs"]
pub(crate) mod conformance;
#[allow(dead_code, unused_imports)]
#[path = "consumption/mod.rs"]
pub(crate) mod consumption;
#[allow(dead_code, unused_imports)]
#[path = "denials/mod.rs"]
pub(crate) mod denials;
#[allow(dead_code, unused_imports)]
#[path = "evaluation/mod.rs"]
pub(crate) mod evaluation;
#[allow(dead_code, unused_imports)]
#[path = "reference/mod.rs"]
pub(crate) mod reference;
#[allow(dead_code, unused_imports)]
#[path = "resources/mod.rs"]
pub(crate) mod resources;
#[allow(dead_code, unused_imports)]
#[path = "stepping/mod.rs"]
pub(crate) mod stepping;
#[allow(dead_code, unused_imports)]
#[path = "vectors/mod.rs"]
pub(crate) mod vectors;
