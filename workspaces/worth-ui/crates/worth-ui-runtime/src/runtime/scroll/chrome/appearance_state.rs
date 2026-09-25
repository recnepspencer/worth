//! The Hover and Pressed classes one chrome part presents.
//!
//! Chrome needs no new appearance axis. `Hover` and `Pressed` already carry the
//! five classes a scrollbar reaches, and the declared roles partition on
//! exactly those two axes. This file is the single place chrome's interaction
//! posture becomes those classes, so a track and the thumb it holds cannot
//! disagree about whether the bar is being dragged.
//!
//! A drag that has left the gutter is `PressedCapturedOutside`, which is why
//! releasing outside the track still ends a held appearance rather than a
//! hovered one.

use worth_ui_dsl::UiAppearanceAxisClass;

/// Where the pointer is with respect to the scrollbar it is dragging.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeDragPosture {
    /// The pointer is still over the gutter it grabbed the thumb in.
    InsideGutter,
    /// The pointer has been dragged off the gutter and capture holds it.
    OutsideGutter,
}

/// The classes one part presents this frame, on the two axes its declared role
/// partitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChromeAppearanceState {
    hover: UiAppearanceAxisClass,
    pressed: UiAppearanceAxisClass,
}

impl UiScrollChromeAppearanceState {
    /// The classes `part` presents, given which part the pointer is hovering on
    /// this axis and whether this axis's thumb is being dragged.
    ///
    /// A drag owns both parts of its own axis: the track a thumb is traveling
    /// in is as held as the thumb is. Hover is decided by the pointer's actual
    /// location, so a drag that leaves the gutter reports the pointer outside
    /// while still reporting the press captured.
    pub(crate) fn resolve(
        part: super::UiScrollChromePart,
        hovered: Option<super::UiScrollChromePart>,
        drag: Option<UiScrollChromeDragPosture>,
    ) -> Self {
        match drag {
            Some(UiScrollChromeDragPosture::InsideGutter) => Self {
                hover: UiAppearanceAxisClass::Hovered,
                pressed: UiAppearanceAxisClass::PressedArmedInside,
            },
            Some(UiScrollChromeDragPosture::OutsideGutter) => Self {
                hover: UiAppearanceAxisClass::HoverOutside,
                pressed: UiAppearanceAxisClass::PressedCapturedOutside,
            },
            None => Self {
                hover: if hovered == Some(part) {
                    UiAppearanceAxisClass::Hovered
                } else {
                    UiAppearanceAxisClass::HoverOutside
                },
                pressed: UiAppearanceAxisClass::PressedIdle,
            },
        }
    }

    #[cfg(test)]
    pub(crate) const fn hover(self) -> UiAppearanceAxisClass {
        self.hover
    }

    #[cfg(test)]
    pub(crate) const fn pressed(self) -> UiAppearanceAxisClass {
        self.pressed
    }

    /// The two classes in axis order, as a declared partition consumes them.
    pub(crate) const fn classes(
        self,
    ) -> [(worth_ui_dsl::UiAppearanceStateAxis, UiAppearanceAxisClass); 2] {
        [
            (worth_ui_dsl::UiAppearanceStateAxis::Hover, self.hover),
            (worth_ui_dsl::UiAppearanceStateAxis::Pressed, self.pressed),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::super::UiScrollChromePart;
    use super::*;

    /// A resting bar is neither hovered nor pressed, on either part.
    #[test]
    fn an_untouched_scrollbar_rests_on_both_parts() {
        for part in UiScrollChromePart::PAINT_ORDER {
            let state = UiScrollChromeAppearanceState::resolve(part, None, None);
            assert_eq!(state.hover(), UiAppearanceAxisClass::HoverOutside);
            assert_eq!(state.pressed(), UiAppearanceAxisClass::PressedIdle);
        }
    }

    /// Hovering the thumb hovers the thumb only. The track under it is not the
    /// part the pointer is on, so it keeps its resting hover class.
    #[test]
    fn hovering_one_part_leaves_the_other_part_resting() {
        let thumb = UiScrollChromeAppearanceState::resolve(
            UiScrollChromePart::Thumb,
            Some(UiScrollChromePart::Thumb),
            None,
        );
        let track = UiScrollChromeAppearanceState::resolve(
            UiScrollChromePart::Track,
            Some(UiScrollChromePart::Thumb),
            None,
        );
        assert_eq!(thumb.hover(), UiAppearanceAxisClass::Hovered);
        assert_eq!(track.hover(), UiAppearanceAxisClass::HoverOutside);
        assert_eq!(thumb.pressed(), UiAppearanceAxisClass::PressedIdle);
    }

    /// A drag holds the whole scrollbar, not just the bar being moved.
    #[test]
    fn a_drag_inside_the_gutter_holds_both_parts_of_its_axis() {
        for part in UiScrollChromePart::PAINT_ORDER {
            let state = UiScrollChromeAppearanceState::resolve(
                part,
                Some(UiScrollChromePart::Thumb),
                Some(UiScrollChromeDragPosture::InsideGutter),
            );
            assert_eq!(state.pressed(), UiAppearanceAxisClass::PressedArmedInside);
            assert_eq!(state.hover(), UiAppearanceAxisClass::Hovered);
        }
    }

    /// Dragging off the gutter keeps the press and loses the hover. That pair
    /// is exactly what the declared roles resolve to the held tone.
    #[test]
    fn a_drag_that_leaves_the_gutter_is_captured_outside() {
        let state = UiScrollChromeAppearanceState::resolve(
            UiScrollChromePart::Thumb,
            None,
            Some(UiScrollChromeDragPosture::OutsideGutter),
        );
        assert_eq!(
            state.pressed(),
            UiAppearanceAxisClass::PressedCapturedOutside
        );
        assert_eq!(state.hover(), UiAppearanceAxisClass::HoverOutside);
    }

    /// The pair is reported on the two axes the declared roles partition, and
    /// on no others.
    #[test]
    fn the_reported_classes_name_the_hover_and_pressed_axes() {
        let state = UiScrollChromeAppearanceState::resolve(UiScrollChromePart::Track, None, None);
        assert_eq!(
            state.classes(),
            [
                (
                    worth_ui_dsl::UiAppearanceStateAxis::Hover,
                    UiAppearanceAxisClass::HoverOutside
                ),
                (
                    worth_ui_dsl::UiAppearanceStateAxis::Pressed,
                    UiAppearanceAxisClass::PressedIdle
                ),
            ]
        );
    }
}
