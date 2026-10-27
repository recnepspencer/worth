//! Registration selection at the native prerequisite boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvariantStage {
    Native,
    Custom {
        packet_offset: usize,
    },
    /// Low-level planning courts inspect the complete registration inventory.
    #[cfg(test)]
    All,
}

impl InvariantStage {
    pub(super) fn native(self) -> bool {
        !matches!(self, Self::Custom { .. })
    }

    pub(super) fn custom(self) -> bool {
        !matches!(self, Self::Native)
    }

    pub(super) fn packet_offset(self) -> usize {
        match self {
            Self::Custom { packet_offset } => packet_offset,
            _ => 0,
        }
    }
}
