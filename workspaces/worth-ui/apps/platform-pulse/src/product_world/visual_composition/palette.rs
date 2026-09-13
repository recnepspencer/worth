use worth_ui::facade::declaration::{
    ThemeTokenAlias, ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource,
    ThemeTokenValue, UiThemeColor,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseRgba([u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulsePaletteRole {
    Canvas,
    RaisedSurface,
    ElevatedSurface,
    StructuralRule,
    PrimaryText,
    SecondaryText,
    PrincipalAccent,
    ActionFill,
    ActionText,
    Positive,
    Caution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformPulseSourceSignalRole {
    SourceSignalBlue,
    SourceSignalGreen,
}

impl PlatformPulsePaletteRole {
    pub const ALL: [Self; 11] = [
        Self::Canvas,
        Self::RaisedSurface,
        Self::ElevatedSurface,
        Self::StructuralRule,
        Self::PrimaryText,
        Self::SecondaryText,
        Self::PrincipalAccent,
        Self::ActionFill,
        Self::ActionText,
        Self::Positive,
        Self::Caution,
    ];

    pub const fn authored_rgba(self) -> PlatformPulseRgba {
        PlatformPulseRgba(match self {
            Self::Canvas => [0x0B, 0x0F, 0x14, 0xFF],
            Self::RaisedSurface => [0x11, 0x16, 0x1C, 0xFF],
            Self::ElevatedSurface => [0x17, 0x1D, 0x25, 0xFF],
            Self::StructuralRule => [0x5F, 0x69, 0x77, 0xFF],
            Self::PrimaryText => [0xF2, 0xF4, 0xF7, 0xFF],
            Self::SecondaryText => [0xA1, 0xA9, 0xB4, 0xFF],
            Self::PrincipalAccent => [0xAC, 0x67, 0xF2, 0xFF],
            Self::ActionFill => [0x94, 0x40, 0xD4, 0xFF],
            Self::ActionText => [0xFA, 0xFB, 0xFC, 0xFF],
            Self::Positive => [0x5C, 0xC9, 0x78, 0xFF],
            Self::Caution => [0xE0, 0xAD, 0x62, 0xFF],
        })
    }

    pub const fn authored_identity(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::RaisedSurface => "raised-surface",
            Self::ElevatedSurface => "elevated-surface",
            Self::StructuralRule => "structural-rule",
            Self::PrimaryText => "primary-text",
            Self::SecondaryText => "secondary-text",
            Self::PrincipalAccent => "principal-accent",
            Self::ActionFill => "action-fill",
            Self::ActionText => "action-text",
            Self::Positive => "positive",
            Self::Caution => "caution",
        }
    }

    pub const fn authored_hex(self) -> &'static str {
        match self {
            Self::Canvas => "#0B0F14",
            Self::RaisedSurface => "#11161C",
            Self::ElevatedSurface => "#171D25",
            Self::StructuralRule => "#5F6977",
            Self::PrimaryText => "#F2F4F7",
            Self::SecondaryText => "#A1A9B4",
            Self::PrincipalAccent => "#AC67F2",
            Self::ActionFill => "#9440D4",
            Self::ActionText => "#FAFBFC",
            Self::Positive => "#5CC978",
            Self::Caution => "#E0AD62",
        }
    }

    const fn token_segment(self) -> &'static str {
        match self {
            Self::Canvas => "canvas",
            Self::RaisedSurface => "raised_surface",
            Self::ElevatedSurface => "elevated_surface",
            Self::StructuralRule => "structural_rule",
            Self::PrimaryText => "primary_text",
            Self::SecondaryText => "secondary_text",
            Self::PrincipalAccent => "principal_accent",
            Self::ActionFill => "action_fill",
            Self::ActionText => "action_text",
            Self::Positive => "positive",
            Self::Caution => "caution",
        }
    }

    pub fn token_id(self) -> ThemeTokenId {
        ThemeTokenId::new(format!("theme.platform_pulse.{}", self.token_segment()))
            .expect("Pulse palette roles are valid token identities")
    }

    fn palette_token_id(self) -> ThemeTokenId {
        ThemeTokenId::new(format!(
            "theme.platform_pulse.palette.{}",
            self.token_segment()
        ))
        .expect("Pulse base palette roles are valid token identities")
    }

    pub fn token_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::define(
            self.palette_token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                UiThemeColor::parse(self.authored_hex())
                    .expect("Pulse palette values are valid authored colors"),
            ),
        )
    }

    pub fn source_alias_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::alias(
            self.token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenAlias::to(self.palette_token_id()),
        )
    }
}

impl PlatformPulseSourceSignalRole {
    pub const ALL: [Self; 2] = [Self::SourceSignalBlue, Self::SourceSignalGreen];

    pub const fn authored_rgba(self) -> PlatformPulseRgba {
        PlatformPulseRgba(match self {
            Self::SourceSignalBlue => [0x2F, 0x81, 0xF7, 0xFF],
            Self::SourceSignalGreen => [0x3F, 0xB9, 0x50, 0xFF],
        })
    }

    pub const fn authored_identity(self) -> &'static str {
        match self {
            Self::SourceSignalBlue => "source-signal-blue",
            Self::SourceSignalGreen => "source-signal-green",
        }
    }

    pub fn token_id(self) -> ThemeTokenId {
        let segment = match self {
            Self::SourceSignalBlue => "blue",
            Self::SourceSignalGreen => "green",
        };
        ThemeTokenId::new(format!("theme.platform_pulse.{segment}"))
            .expect("Pulse source-signal roles are valid token identities")
    }

    pub fn token_descriptor(self) -> ThemeTokenDescriptor {
        ThemeTokenDescriptor::define(
            self.token_id(),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            ThemeTokenValue::color(
                UiThemeColor::parse(match self {
                    Self::SourceSignalBlue => "#2F81F7",
                    Self::SourceSignalGreen => "#3FB950",
                })
                .expect("Pulse source-signal values are valid authored colors"),
            ),
        )
    }
}

impl PlatformPulseRgba {
    pub const fn channels(self) -> [u8; 4] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn authored_roles_build_distinct_real_application_tokens() {
        let descriptors = PlatformPulsePaletteRole::ALL
            .into_iter()
            .map(PlatformPulsePaletteRole::token_descriptor)
            .collect::<Vec<_>>();
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.id().as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            PlatformPulsePaletteRole::ALL.len()
        );
        assert!(descriptors
            .iter()
            .all(|descriptor| descriptor.value().is_some()));
    }
}
