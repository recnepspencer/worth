use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LocatorAuthority {
    Authoritative,
    Derived,
    Projected,
    SupportOnly,
    Planned,
    ReceiptBearing,
}
