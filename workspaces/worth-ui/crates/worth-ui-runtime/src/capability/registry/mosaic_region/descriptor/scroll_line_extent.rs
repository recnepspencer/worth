/// Why a declared scroll line extent was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollLineExtentDenial {
    /// A line of zero extent gives every wheel notch no travel at all.
    LogicalPointsAreZero,
    /// The line is taller than [`UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS`],
    /// beyond which one notch would jump further than any viewport this
    /// declaration can serve.
    LogicalPointsExceedMaximum,
}

/// The tallest line a scroll region may declare, in logical points.
pub const UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS: u16 = 512;

/// What one line of a scroll region's content is worth, in logical points.
///
/// The host reports wheel travel in lines and states the line count it applied;
/// it does not know how tall a line is. This declaration is where that answer
/// lives, beside the region whose content defines it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiScrollLineExtent {
    logical_points: u16,
}

impl UiScrollLineExtent {
    pub fn logical_points(points: u16) -> Result<Self, UiScrollLineExtentDenial> {
        if points == 0 {
            return Err(UiScrollLineExtentDenial::LogicalPointsAreZero);
        }
        if points > UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS {
            return Err(UiScrollLineExtentDenial::LogicalPointsExceedMaximum);
        }
        Ok(Self {
            logical_points: points,
        })
    }

    pub const fn logical_points_value(self) -> u16 {
        self.logical_points
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiScrollLineExtent, UiScrollLineExtentDenial, UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS,
    };

    /// A line has an extent, and that extent stays within a named maximum.
    #[test]
    fn a_line_extent_is_positive_and_within_the_named_maximum() {
        assert_eq!(
            UiScrollLineExtent::logical_points(0),
            Err(UiScrollLineExtentDenial::LogicalPointsAreZero)
        );
        assert_eq!(
            UiScrollLineExtent::logical_points(UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS + 1),
            Err(UiScrollLineExtentDenial::LogicalPointsExceedMaximum)
        );
        assert_eq!(
            UiScrollLineExtent::logical_points(20).map(UiScrollLineExtent::logical_points_value),
            Ok(20)
        );
        assert_eq!(
            UiScrollLineExtent::logical_points(UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS)
                .map(UiScrollLineExtent::logical_points_value),
            Ok(UI_SCROLL_LINE_EXTENT_MAXIMUM_LOGICAL_POINTS)
        );
    }
}
