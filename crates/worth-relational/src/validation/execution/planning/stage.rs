//! Registration selection at the native prerequisite boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvariantStage {
    Native,
    NativeAfterBoundary {
        packet_offset: usize,
    },
    CustomAfterBoundary {
        packet_offset: usize,
    },
    Custom {
        packet_offset: usize,
    },
    /// Low-level planning courts inspect the complete registration inventory.
    #[cfg(test)]
    All,
}

impl InvariantStage {
    pub(super) fn registration_points(
        self,
        point: crate::validation::data::InvariantExecutionPoint,
    ) -> [Option<crate::validation::data::InvariantExecutionPoint>; 2] {
        use crate::validation::data::InvariantExecutionPoint as Point;
        match self {
            Self::NativeAfterBoundary { .. } => [
                Some(Point::MutationSensitive),
                Some(Point::SnapshotPublication),
            ],
            Self::CustomAfterBoundary { .. } => [
                Some(Point::MutationSensitive),
                Some(Point::SnapshotPublication),
            ],
            _ => [Some(point), None],
        }
    }
    pub(super) fn native(self) -> bool {
        !matches!(self, Self::Custom { .. } | Self::CustomAfterBoundary { .. })
    }

    pub(super) fn custom(self) -> bool {
        !matches!(self, Self::Native | Self::NativeAfterBoundary { .. })
    }

    pub(super) fn packet_offset(self) -> usize {
        match self {
            Self::Custom { packet_offset }
            | Self::CustomAfterBoundary { packet_offset }
            | Self::NativeAfterBoundary { packet_offset } => packet_offset,
            _ => 0,
        }
    }
}
