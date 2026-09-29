use sha2::Digest;
use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{UiFontSlant, UiQualifiedFontFamilyIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiFontFamilyStack(Box<[UiQualifiedFontFamilyIdentity]>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiTextFaceRequest {
    weight: u16,
    width_milli_percent: u32,
    slant: UiFontSlant,
}

impl UiFontFamilyStack {
    pub fn new(families: Box<[UiQualifiedFontFamilyIdentity]>) -> Option<Self> {
        if families.is_empty()
            || families
                .iter()
                .enumerate()
                .any(|(index, family)| families[..index].contains(family))
        {
            return None;
        }
        Some(Self(families))
    }

    pub fn profile_sans() -> Self {
        Self(Box::new([profile_family_identity("noto-sans")]))
    }

    pub fn families(&self) -> &[UiQualifiedFontFamilyIdentity] {
        &self.0
    }
}

impl UiTextFaceRequest {
    pub const fn new(weight: u16, width_milli_percent: u32, slant: UiFontSlant) -> Option<Self> {
        if weight == 0
            || weight > 1_000
            || width_milli_percent < 50_000
            || width_milli_percent > 200_000
        {
            return None;
        }
        Some(Self {
            weight,
            width_milli_percent,
            slant,
        })
    }

    pub const fn regular() -> Self {
        Self {
            weight: 400,
            width_milli_percent: 100_000,
            slant: UiFontSlant::Upright,
        }
    }

    pub const fn weight(self) -> u16 {
        self.weight
    }
    pub const fn width_milli_percent(self) -> u32 {
        self.width_milli_percent
    }
    pub const fn slant(self) -> UiFontSlant {
        self.slant
    }
}

pub(crate) fn profile_family_identity(name: &str) -> UiQualifiedFontFamilyIdentity {
    let mut hash = UiCountedSha256::new();
    hash.update(b"worth-ui-profile-family-v2\0");
    hash.update(name.as_bytes());
    UiQualifiedFontFamilyIdentity::from_text_mechanics(hash.finalize().into())
}
