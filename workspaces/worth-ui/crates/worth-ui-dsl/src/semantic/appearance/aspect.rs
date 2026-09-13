#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiAppearanceAspect {
    Background,
    Foreground,
    Border,
    Radius,
    Opacity,
    Outline,
}

impl UiAppearanceAspect {
    pub fn accepts_value_kind(self, kind: super::UiThemeValueKind) -> bool {
        kind == self.value_kind()
            || (self == Self::Background && kind == super::UiThemeValueKind::LinearGradient)
    }

    pub const fn value_kind(self) -> super::UiThemeValueKind {
        match self {
            Self::Background | Self::Foreground => super::UiThemeValueKind::Color,
            Self::Border => super::UiThemeValueKind::SolidStroke,
            Self::Radius => super::UiThemeValueKind::CornerRadii,
            Self::Opacity => super::UiThemeValueKind::Opacity,
            Self::Outline => super::UiThemeValueKind::SolidOutline,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceAspectContract {
    applicability: UiAppearanceAspectApplicability,
    required: Box<[UiAppearanceAspect]>,
    optional: Box<[UiAppearanceAspect]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiAppearanceAspectApplicability {
    Component,
    Backdrop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceAspectContractDenial {
    DuplicateAspect,
    RequiredAndOptional,
}

impl UiAppearanceAspectContract {
    pub fn component(
        required: impl IntoIterator<Item = UiAppearanceAspect>,
        optional: impl IntoIterator<Item = UiAppearanceAspect>,
    ) -> Result<Self, UiAppearanceAspectContractDenial> {
        Self::new(
            UiAppearanceAspectApplicability::Component,
            required,
            optional,
        )
    }

    pub fn backdrop() -> Self {
        Self {
            applicability: UiAppearanceAspectApplicability::Backdrop,
            required: Box::new([UiAppearanceAspect::Background, UiAppearanceAspect::Opacity]),
            optional: Box::new([]),
        }
    }

    fn new(
        applicability: UiAppearanceAspectApplicability,
        required: impl IntoIterator<Item = UiAppearanceAspect>,
        optional: impl IntoIterator<Item = UiAppearanceAspect>,
    ) -> Result<Self, UiAppearanceAspectContractDenial> {
        let required = canonical(required)?;
        let optional = canonical(optional)?;
        if required.iter().any(|aspect| optional.contains(aspect)) {
            return Err(UiAppearanceAspectContractDenial::RequiredAndOptional);
        }
        Ok(Self {
            applicability,
            required,
            optional,
        })
    }

    pub fn required(&self) -> &[UiAppearanceAspect] {
        &self.required
    }

    pub const fn applicability(&self) -> UiAppearanceAspectApplicability {
        self.applicability
    }

    pub fn optional(&self) -> &[UiAppearanceAspect] {
        &self.optional
    }

    pub fn admits(&self, aspect: UiAppearanceAspect) -> bool {
        self.required.contains(&aspect) || self.optional.contains(&aspect)
    }
}

fn canonical(
    values: impl IntoIterator<Item = UiAppearanceAspect>,
) -> Result<Box<[UiAppearanceAspect]>, UiAppearanceAspectContractDenial> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(UiAppearanceAspectContractDenial::DuplicateAspect);
    }
    Ok(values.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_gate_zero_aspect_has_its_exact_value_kind() {
        assert_eq!(
            UiAppearanceAspect::Background.value_kind(),
            super::super::UiThemeValueKind::Color
        );
        assert_eq!(
            UiAppearanceAspect::Foreground.value_kind(),
            super::super::UiThemeValueKind::Color
        );
        assert_eq!(
            UiAppearanceAspect::Border.value_kind(),
            super::super::UiThemeValueKind::SolidStroke
        );
        assert_eq!(
            UiAppearanceAspect::Radius.value_kind(),
            super::super::UiThemeValueKind::CornerRadii
        );
        assert_eq!(
            UiAppearanceAspect::Opacity.value_kind(),
            super::super::UiThemeValueKind::Opacity
        );
        assert_eq!(
            UiAppearanceAspect::Outline.value_kind(),
            super::super::UiThemeValueKind::SolidOutline
        );
    }
}
