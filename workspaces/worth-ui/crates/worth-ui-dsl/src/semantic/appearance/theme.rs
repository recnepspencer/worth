#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiThemeColor([u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeColorParseDenial {
    UnsupportedLength,
    MissingHash,
    NonAsciiHex,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiThemeOpacity(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeOpacityDenial {
    ZeroDenominator,
    OutsideUnitInterval,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiLogicalLength(i32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiThemeCornerRadii {
    top_left: UiLogicalLength,
    top_right: UiLogicalLength,
    bottom_right: UiLogicalLength,
    bottom_left: UiLogicalLength,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiThemeSolidStroke {
    color: UiThemeColor,
    width: UiLogicalLength,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiThemeOutline {
    stroke: UiThemeSolidStroke,
    offset: UiLogicalLength,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiThemeValueKind {
    Color,
    Opacity,
    LogicalLength,
    CornerRadii,
    SolidStroke,
    SolidOutline,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiThemeValue {
    Color(UiThemeColor),
    Opacity(UiThemeOpacity),
    LogicalLength(UiLogicalLength),
    CornerRadii(UiThemeCornerRadii),
    SolidStroke(UiThemeSolidStroke),
    SolidOutline(UiThemeOutline),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiThemeSlotIdentity(Box<str>);

impl UiThemeColor {
    pub const fn from_channels(channels: [u8; 4]) -> Self {
        Self(channels)
    }

    pub fn parse(value: &str) -> Result<Self, UiThemeColorParseDenial> {
        if !value.starts_with('#') {
            return Err(UiThemeColorParseDenial::MissingHash);
        }
        if value.len() != 7 && value.len() != 9 {
            return Err(UiThemeColorParseDenial::UnsupportedLength);
        }
        let mut channels = [0_u8; 4];
        for (index, channel) in channels.iter_mut().take(3).enumerate() {
            *channel = parse_hex_byte(&value.as_bytes()[1 + index * 2..3 + index * 2])?;
        }
        channels[3] = if value.len() == 9 {
            parse_hex_byte(&value.as_bytes()[7..9])?
        } else {
            u8::MAX
        };
        Ok(Self(channels))
    }

    pub const fn channels(self) -> [u8; 4] {
        self.0
    }
}

fn parse_hex_byte(bytes: &[u8]) -> Result<u8, UiThemeColorParseDenial> {
    let high = hex_digit(bytes[0]).ok_or(UiThemeColorParseDenial::NonAsciiHex)?;
    let low = hex_digit(bytes[1]).ok_or(UiThemeColorParseDenial::NonAsciiHex)?;
    Ok(high * 16 + low)
}

const fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

impl UiThemeOpacity {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u16::MAX);

    pub const fn from_units(units: u16) -> Self {
        Self(units)
    }

    pub fn from_ratio(numerator: u64, denominator: u64) -> Result<Self, UiThemeOpacityDenial> {
        if denominator == 0 {
            return Err(UiThemeOpacityDenial::ZeroDenominator);
        }
        if numerator > denominator {
            return Err(UiThemeOpacityDenial::OutsideUnitInterval);
        }
        Ok(Self(round_ratio_to_even(
            u128::from(numerator) * u128::from(u16::MAX),
            u128::from(denominator),
        ) as u16))
    }

    pub const fn units(self) -> u16 {
        self.0
    }
}

fn round_ratio_to_even(numerator: u128, denominator: u128) -> u128 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    match (remainder * 2).cmp(&denominator) {
        std::cmp::Ordering::Less => quotient,
        std::cmp::Ordering::Greater => quotient + 1,
        std::cmp::Ordering::Equal if quotient % 2 == 1 => quotient + 1,
        std::cmp::Ordering::Equal => quotient,
    }
}

impl UiLogicalLength {
    pub const SUBPIXELS_PER_LOGICAL_POINT: i32 = 1_000;

    pub const fn new(subpixels: i32) -> Self {
        Self(subpixels)
    }

    pub const fn nonnegative(subpixels: i32) -> Option<Self> {
        if subpixels < 0 {
            None
        } else {
            Some(Self(subpixels))
        }
    }

    pub const fn subpixels(self) -> i32 {
        self.0
    }
}

impl UiThemeCornerRadii {
    pub const fn new(
        top_left: UiLogicalLength,
        top_right: UiLogicalLength,
        bottom_right: UiLogicalLength,
        bottom_left: UiLogicalLength,
    ) -> Option<Self> {
        if top_left.0 < 0 || top_right.0 < 0 || bottom_right.0 < 0 || bottom_left.0 < 0 {
            None
        } else {
            Some(Self {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            })
        }
    }

    pub const fn corners(self) -> [UiLogicalLength; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }
}

impl UiThemeSolidStroke {
    pub const fn new(color: UiThemeColor, width: UiLogicalLength) -> Option<Self> {
        if width.0 < 0 {
            None
        } else {
            Some(Self { color, width })
        }
    }

    pub const fn color(self) -> UiThemeColor {
        self.color
    }
    pub const fn width(self) -> UiLogicalLength {
        self.width
    }
}

impl UiThemeOutline {
    pub const fn new(stroke: UiThemeSolidStroke, offset: UiLogicalLength) -> Option<Self> {
        if offset.0 < 0 {
            None
        } else {
            Some(Self { stroke, offset })
        }
    }

    pub const fn stroke(self) -> UiThemeSolidStroke {
        self.stroke
    }
    pub const fn offset(self) -> UiLogicalLength {
        self.offset
    }
}

impl UiThemeValue {
    pub const fn kind(self) -> UiThemeValueKind {
        match self {
            Self::Color(_) => UiThemeValueKind::Color,
            Self::Opacity(_) => UiThemeValueKind::Opacity,
            Self::LogicalLength(_) => UiThemeValueKind::LogicalLength,
            Self::CornerRadii(_) => UiThemeValueKind::CornerRadii,
            Self::SolidStroke(_) => UiThemeValueKind::SolidStroke,
            Self::SolidOutline(_) => UiThemeValueKind::SolidOutline,
        }
    }
}

impl UiThemeSlotIdentity {
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty() && value.len() <= 128 && value.is_ascii()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_parses_once_into_canonical_bytes() {
        assert_eq!(
            UiThemeColor::parse("#Aa00fF").unwrap().channels(),
            [170, 0, 255, 255]
        );
        assert_eq!(
            UiThemeColor::parse("#aa00ff40").unwrap().channels(),
            [170, 0, 255, 64]
        );
        assert_eq!(
            UiThemeColor::parse("aa00ff"),
            Err(UiThemeColorParseDenial::MissingHash)
        );
    }

    #[test]
    fn opacity_authoring_quantizes_exact_ratios_to_nearest_even_units() {
        assert_eq!(UiThemeOpacity::from_ratio(1, 2).unwrap().units(), 32_768);
        assert_eq!(UiThemeOpacity::from_ratio(40, 64).unwrap().units(), 40_959);
        assert_eq!(
            UiThemeOpacity::from_ratio(u64::MAX - 1, u64::MAX)
                .unwrap()
                .units(),
            u16::MAX
        );
    }

    #[test]
    fn radii_preserve_css_corner_order_and_negative_geometry_denies() {
        let length = |value| UiLogicalLength::new(value);
        let radii = UiThemeCornerRadii::new(length(1), length(2), length(3), length(4)).unwrap();
        assert_eq!(
            radii.corners().map(UiLogicalLength::subpixels),
            [1, 2, 3, 4]
        );
        for negative_corner in 0..4 {
            let mut corners = [length(0); 4];
            corners[negative_corner] = length(-1);
            assert!(
                UiThemeCornerRadii::new(corners[0], corners[1], corners[2], corners[3]).is_none()
            );
        }
        let color = UiThemeColor::from_channels([1, 2, 3, 4]);
        assert!(UiThemeSolidStroke::new(color, length(-1)).is_none());
        let stroke = UiThemeSolidStroke::new(color, length(1)).unwrap();
        assert!(UiThemeOutline::new(stroke, length(-1)).is_none());
    }
}
