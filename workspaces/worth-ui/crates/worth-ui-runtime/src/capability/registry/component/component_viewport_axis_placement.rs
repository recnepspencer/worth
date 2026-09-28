/// Where a viewport axis placement starts and how far it extends along an
/// axis, in logical points, before any caller's posture on a span that falls
/// outside the axis: the start may be negative and the extent zero or
/// negative when the axis is too short for the placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ComponentViewportAxisSpan {
    pub(crate) start: f32,
    pub(crate) extent: f32,
}

/// One authored axis of a component whose bounds are resolved directly from
/// the admitted logical viewport.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComponentViewportAxisPlacement {
    FixedFromStart {
        start_logical_points: u16,
        extent_logical_points: u16,
    },
    StretchBetween {
        start_logical_points: u16,
        end_logical_points: u16,
    },
    /// Keeps its start inset and stretches toward the end inset, but never
    /// past its maximum extent: a card that holds its authored height where
    /// there is room and gives way to the insets where there is not.
    BoundedStretch {
        start_logical_points: u16,
        end_logical_points: u16,
        maximum_logical_points: u16,
    },
    FixedFromEnd {
        end_logical_points: u16,
        extent_logical_points: u16,
    },
}

impl ComponentViewportAxisPlacement {
    pub const fn fixed_from_start(
        start_logical_points: u16,
        extent_logical_points: u16,
    ) -> Option<Self> {
        if extent_logical_points == 0 {
            return None;
        }
        Some(Self::FixedFromStart {
            start_logical_points,
            extent_logical_points,
        })
    }

    pub const fn stretch_between(start_logical_points: u16, end_logical_points: u16) -> Self {
        Self::StretchBetween {
            start_logical_points,
            end_logical_points,
        }
    }

    pub const fn bounded_stretch(
        start_logical_points: u16,
        end_logical_points: u16,
        maximum_logical_points: u16,
    ) -> Option<Self> {
        if maximum_logical_points == 0 {
            return None;
        }
        Some(Self::BoundedStretch {
            start_logical_points,
            end_logical_points,
            maximum_logical_points,
        })
    }

    pub const fn fixed_from_end(
        end_logical_points: u16,
        extent_logical_points: u16,
    ) -> Option<Self> {
        if extent_logical_points == 0 {
            return None;
        }
        Some(Self::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        })
    }

    /// The span this placement takes along an axis `available` points long.
    pub(crate) fn span(self, available: f32) -> ComponentViewportAxisSpan {
        let (start, extent) = match self {
            Self::FixedFromStart {
                start_logical_points,
                extent_logical_points,
            } => (
                f32::from(start_logical_points),
                f32::from(extent_logical_points),
            ),
            Self::StretchBetween {
                start_logical_points,
                end_logical_points,
            } => {
                let start = f32::from(start_logical_points);
                (start, available - start - f32::from(end_logical_points))
            }
            Self::BoundedStretch {
                start_logical_points,
                end_logical_points,
                maximum_logical_points,
            } => {
                let start = f32::from(start_logical_points);
                let between = available - start - f32::from(end_logical_points);
                (start, between.min(f32::from(maximum_logical_points)))
            }
            Self::FixedFromEnd {
                end_logical_points,
                extent_logical_points,
            } => {
                let extent = f32::from(extent_logical_points);
                (available - f32::from(end_logical_points) - extent, extent)
            }
        };
        ComponentViewportAxisSpan { start, extent }
    }

    pub(crate) fn digest_basis(self) -> String {
        match self {
            Self::FixedFromStart {
                start_logical_points,
                extent_logical_points,
            } => format!("fixed-from-start:{start_logical_points}:{extent_logical_points}"),
            Self::StretchBetween {
                start_logical_points,
                end_logical_points,
            } => format!("stretch-between:{start_logical_points}:{end_logical_points}"),
            Self::BoundedStretch {
                start_logical_points,
                end_logical_points,
                maximum_logical_points,
            } => format!(
                "bounded-stretch:{start_logical_points}:{end_logical_points}:{maximum_logical_points}"
            ),
            Self::FixedFromEnd {
                end_logical_points,
                extent_logical_points,
            } => format!("fixed-from-end:{end_logical_points}:{extent_logical_points}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentViewportAxisPlacement, ComponentViewportAxisSpan};

    #[test]
    fn fixed_axes_reject_empty_extent_and_keep_direction_in_identity() {
        assert!(ComponentViewportAxisPlacement::fixed_from_start(24, 0).is_none());
        assert!(ComponentViewportAxisPlacement::fixed_from_end(24, 0).is_none());
        assert!(ComponentViewportAxisPlacement::bounded_stretch(24, 24, 0).is_none());
        assert_ne!(
            ComponentViewportAxisPlacement::fixed_from_start(24, 56)
                .unwrap()
                .digest_basis(),
            ComponentViewportAxisPlacement::fixed_from_end(24, 56)
                .unwrap()
                .digest_basis(),
        );
    }

    /// A bounded stretch keeps its maximum where the insets leave more, and
    /// gives way to the insets where they leave less.
    #[test]
    fn a_bounded_stretch_holds_its_maximum_until_the_insets_leave_less() {
        let card = ComponentViewportAxisPlacement::bounded_stretch(36, 36, 444).unwrap();
        assert_eq!(
            card.span(1_024.0),
            ComponentViewportAxisSpan {
                start: 36.0,
                extent: 444.0
            }
        );
        assert_eq!(
            card.span(516.0),
            ComponentViewportAxisSpan {
                start: 36.0,
                extent: 444.0
            }
        );
        assert_eq!(
            card.span(400.0),
            ComponentViewportAxisSpan {
                start: 36.0,
                extent: 328.0
            }
        );
        assert_ne!(
            card.digest_basis(),
            ComponentViewportAxisPlacement::stretch_between(36, 36).digest_basis(),
        );
    }
}
