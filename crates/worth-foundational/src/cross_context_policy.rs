use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrossContextPolicy {
    AllowExplicit,
    SchemaControlled,
    Forbid,
}
