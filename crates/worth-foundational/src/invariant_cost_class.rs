use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantCostClass {
    Touched,
    Partition,
    Global,
}

impl InvariantCostClass {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Touched => "touched",
            Self::Partition => "partition",
            Self::Global => "global",
        }
    }
}
