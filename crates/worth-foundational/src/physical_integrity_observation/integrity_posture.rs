use serde::{Deserialize, Serialize};

/// Pure observation outcome; it carries no owner or lifecycle authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalIntegrityPosture {
    Intact,
    Damaged,
    Unsupported,
    Unknown,
    Indeterminate,
}
