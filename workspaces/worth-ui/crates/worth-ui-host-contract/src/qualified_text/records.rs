#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiTextOriginalRange {
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQualifiedTextGraphemeRecord {
    original_range: UiTextOriginalRange,
    bidi_level: u8,
    direction: UiTextDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQualifiedTextWordBoundaryRecord {
    original_boundary: UiTextOriginalRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQualifiedTextGlyphRecord {
    glyph_id: u32,
    original_range: UiTextOriginalRange,
    x_advance_font_units: i32,
    y_advance_font_units: i32,
    x_offset_font_units: i32,
    y_offset_font_units: i32,
    ink_bounds_font_units: super::UiTextFontUnitRect,
}

#[doc(hidden)]
pub struct UiQualifiedTextGlyphInput {
    pub glyph_id: u32,
    pub original_range: UiTextOriginalRange,
    pub x_advance_font_units: i32,
    pub y_advance_font_units: i32,
    pub x_offset_font_units: i32,
    pub y_offset_font_units: i32,
    pub ink_bounds_font_units: super::UiTextFontUnitRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQualifiedTextRunRecord {
    original_range: UiTextOriginalRange,
    glyph_start: u32,
    glyph_end: u32,
    face: super::UiQualifiedFontFaceIdentity,
    script_tag: [u8; 4],
    bidi_level: u8,
    units_per_em: u16,
    style_index: u16,
    ascender_font_units: i16,
    descender_font_units: i16,
    line_gap_font_units: i16,
}

#[doc(hidden)]
pub struct UiQualifiedTextRunInput {
    pub original_range: UiTextOriginalRange,
    pub glyph_start: u32,
    pub glyph_end: u32,
    pub face: super::UiQualifiedFontFaceIdentity,
    pub script_tag: [u8; 4],
    pub bidi_level: u8,
    pub units_per_em: u16,
    pub style_index: u16,
    pub ascender_font_units: i16,
    pub descender_font_units: i16,
    pub line_gap_font_units: i16,
}

impl UiTextOriginalRange {
    pub const fn new(start: u32, end: u32) -> Option<Self> {
        Self::from_text_mechanics(start, end)
    }

    #[doc(hidden)]
    pub const fn from_text_mechanics(start: u32, end: u32) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub const fn start(self) -> u32 {
        self.start
    }

    pub const fn end(self) -> u32 {
        self.end
    }

    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

impl UiQualifiedTextGraphemeRecord {
    #[doc(hidden)]
    pub const fn from_text_mechanics(original_range: UiTextOriginalRange, bidi_level: u8) -> Self {
        Self {
            original_range,
            bidi_level,
            direction: if bidi_level.is_multiple_of(2) {
                UiTextDirection::LeftToRight
            } else {
                UiTextDirection::RightToLeft
            },
        }
    }

    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }

    pub const fn bidi_level(self) -> u8 {
        self.bidi_level
    }

    pub const fn direction(self) -> UiTextDirection {
        self.direction
    }
}

impl UiQualifiedTextWordBoundaryRecord {
    #[doc(hidden)]
    pub const fn from_text_mechanics(original_boundary: UiTextOriginalRange) -> Self {
        assert!(
            original_boundary.is_empty(),
            "a word boundary is an empty original range"
        );
        Self { original_boundary }
    }

    pub const fn original_boundary(self) -> UiTextOriginalRange {
        self.original_boundary
    }
}

impl UiQualifiedTextGlyphRecord {
    #[doc(hidden)]
    pub const fn from_text_mechanics(input: UiQualifiedTextGlyphInput) -> Self {
        Self {
            glyph_id: input.glyph_id,
            original_range: input.original_range,
            x_advance_font_units: input.x_advance_font_units,
            y_advance_font_units: input.y_advance_font_units,
            x_offset_font_units: input.x_offset_font_units,
            y_offset_font_units: input.y_offset_font_units,
            ink_bounds_font_units: input.ink_bounds_font_units,
        }
    }

    pub const fn glyph_id(self) -> u32 {
        self.glyph_id
    }
    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }
    pub const fn x_advance_font_units(self) -> i32 {
        self.x_advance_font_units
    }
    pub const fn y_advance_font_units(self) -> i32 {
        self.y_advance_font_units
    }
    pub const fn x_offset_font_units(self) -> i32 {
        self.x_offset_font_units
    }
    pub const fn y_offset_font_units(self) -> i32 {
        self.y_offset_font_units
    }
    pub const fn ink_bounds_font_units(self) -> super::UiTextFontUnitRect {
        self.ink_bounds_font_units
    }
}

impl UiQualifiedTextRunRecord {
    #[doc(hidden)]
    pub const fn from_text_mechanics(input: UiQualifiedTextRunInput) -> Self {
        Self {
            original_range: input.original_range,
            glyph_start: input.glyph_start,
            glyph_end: input.glyph_end,
            face: input.face,
            script_tag: input.script_tag,
            bidi_level: input.bidi_level,
            units_per_em: input.units_per_em,
            style_index: input.style_index,
            ascender_font_units: input.ascender_font_units,
            descender_font_units: input.descender_font_units,
            line_gap_font_units: input.line_gap_font_units,
        }
    }

    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }
    pub const fn glyph_range(self) -> core::ops::Range<u32> {
        self.glyph_start..self.glyph_end
    }
    pub const fn face(self) -> super::UiQualifiedFontFaceIdentity {
        self.face
    }
    pub const fn script_tag(self) -> [u8; 4] {
        self.script_tag
    }
    pub const fn bidi_level(self) -> u8 {
        self.bidi_level
    }
    pub const fn units_per_em(self) -> u16 {
        self.units_per_em
    }
    pub const fn style_index(self) -> u16 {
        self.style_index
    }
    pub const fn ascender_font_units(self) -> i16 {
        self.ascender_font_units
    }
    pub const fn descender_font_units(self) -> i16 {
        self.descender_font_units
    }
    pub const fn line_gap_font_units(self) -> i16 {
        self.line_gap_font_units
    }
}
