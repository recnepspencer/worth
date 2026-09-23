//! The two painted parts of one scrollbar, and the declared role each takes.
//!
//! A scrollbar is a track and a thumb, and the chrome contract names one
//! registered appearance role for each. Keeping the part a named value rather
//! than a boolean is what lets lowering, hit testing and appearance state all
//! say the same word about the same rectangle.

/// One painted part of one axis's chrome.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiScrollChromePart {
    /// The reserved gutter strip the thumb travels along.
    Track,
    /// The bar whose length reports the viewport proportion and whose position
    /// reports the accepted displayed offset.
    Thumb,
}

impl UiScrollChromePart {
    /// Both parts in paint order: the track is painted first so the thumb sits
    /// on it.
    pub(crate) const PAINT_ORDER: [Self; 2] = [Self::Track, Self::Thumb];

    /// The rectangle this part occupies on one axis of derived chrome.
    pub(crate) const fn rect(
        self,
        facts: super::UiScrollChromeAxisFacts,
    ) -> worth_ui_host_contract::UiMountedCanonicalBox {
        match self {
            Self::Track => facts.track(),
            Self::Thumb => facts.thumb(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The track is painted before the thumb, because the thumb sits on it.
    #[test]
    fn paint_order_puts_the_track_under_the_thumb() {
        assert_eq!(
            UiScrollChromePart::PAINT_ORDER,
            [UiScrollChromePart::Track, UiScrollChromePart::Thumb]
        );
    }
}
