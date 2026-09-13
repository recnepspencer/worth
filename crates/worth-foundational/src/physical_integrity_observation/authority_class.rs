use serde::{Deserialize, Serialize};

/// Descriptive role assigned by the artifact owner after validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalAuthorityClass {
    Authority,
    Derived,
    NonAuthoritativeMetadata,
}
