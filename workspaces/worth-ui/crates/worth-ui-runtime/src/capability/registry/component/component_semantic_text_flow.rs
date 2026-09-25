use std::num::NonZeroU32;

/// How a paragraph fills the width it is allocated: where its lines may
/// break, how many it may take, and what it shows when it runs out. Height is
/// not part of it: a taller or shorter box clips the same shaped lines, so a
/// height-only change never reshapes text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentSemanticTextFlow {
    wrap: worth_ui_text::UiTextWrap,
    overflow: worth_ui_text::UiTextOverflow,
    maximum_lines: Option<NonZeroU32>,
}

impl ComponentSemanticTextFlow {
    /// Breaks between words and keeps every line; the box clips what it
    /// cannot show.
    pub const fn wrapping() -> Self {
        Self::new(
            worth_ui_text::UiTextWrap::UnicodeWord,
            worth_ui_text::UiTextOverflow::Clip,
            None,
        )
    }

    /// One line that ends in an ellipsis when its allocated width cannot hold
    /// it.
    pub const fn single_line_ellipsis() -> Self {
        Self::new(
            worth_ui_text::UiTextWrap::None,
            worth_ui_text::UiTextOverflow::Ellipsis,
            Some(NonZeroU32::MIN),
        )
    }

    /// `maximum_lines` of `None` keeps every line the text needs.
    pub const fn new(
        wrap: worth_ui_text::UiTextWrap,
        overflow: worth_ui_text::UiTextOverflow,
        maximum_lines: Option<NonZeroU32>,
    ) -> Self {
        Self {
            wrap,
            overflow,
            maximum_lines,
        }
    }

    pub const fn wrap(&self) -> worth_ui_text::UiTextWrap {
        self.wrap
    }

    pub const fn overflow(&self) -> worth_ui_text::UiTextOverflow {
        self.overflow
    }

    pub const fn maximum_lines(&self) -> Option<NonZeroU32> {
        self.maximum_lines
    }

    /// The line limit shaping honors: the declared one, or as many lines as
    /// the text profile can record.
    pub(crate) fn line_limit(&self) -> u32 {
        self.maximum_lines.map_or_else(
            || {
                u32::try_from(worth_ui_text::UiGlobalTextProfile::MAX_LINE_RECORDS)
                    .expect("profile line cap fits u32")
            },
            NonZeroU32::get,
        )
    }

    pub(crate) fn digest_basis(&self) -> String {
        format!(
            "wrap:{:?}:overflow:{:?}:lines:{:?}",
            self.wrap, self.overflow, self.maximum_lines
        )
    }
}

impl Default for ComponentSemanticTextFlow {
    fn default() -> Self {
        Self::wrapping()
    }
}
