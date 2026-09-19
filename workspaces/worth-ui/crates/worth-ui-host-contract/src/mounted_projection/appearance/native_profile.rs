use crate::runtime::WorthUiHostCapabilityDigest;

use super::UiHostAppearanceGeometryQualification;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiHostAppearanceMechanicFamily {
    SurfaceFill,
    SurfaceBorder,
    CornerRadii,
    Outline,
    TextRangeForeground,
    PortalSurface,
    Backdrop,
    OverlayOrder,
    PointerAffordance,
    Damage,
    Clip,
}

impl UiHostAppearanceMechanicFamily {
    pub const ALL: [Self; 11] = [
        Self::SurfaceFill,
        Self::SurfaceBorder,
        Self::CornerRadii,
        Self::Outline,
        Self::TextRangeForeground,
        Self::PortalSurface,
        Self::Backdrop,
        Self::OverlayOrder,
        Self::PointerAffordance,
        Self::Damage,
        Self::Clip,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostAppearanceProfilePosture {
    Current,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHostAppearanceProfileContract {
    posture: UiHostAppearanceProfilePosture,
    identity: Box<str>,
    version: u16,
    mechanics: Box<[UiHostAppearanceMechanicFamily]>,
    primary_pointer: Option<super::UiHostPrimaryPointerKind>,
    geometry_qualification: UiHostAppearanceGeometryQualification,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostAppearanceProfileDenial {
    EmptyIdentity,
    DuplicateMechanic,
    MissingRequiredMechanic,
}

impl UiHostAppearanceProfileContract {
    pub fn admit(
        identity: impl Into<Box<str>>,
        version: u16,
        mechanics: impl IntoIterator<Item = UiHostAppearanceMechanicFamily>,
        primary_pointer: Option<super::UiHostPrimaryPointerKind>,
        geometry_qualification: UiHostAppearanceGeometryQualification,
    ) -> Result<Self, UiHostAppearanceProfileDenial> {
        let identity = identity.into();
        if identity.is_empty() {
            return Err(UiHostAppearanceProfileDenial::EmptyIdentity);
        }
        let mut mechanics = mechanics.into_iter().collect::<Vec<_>>();
        mechanics.sort();
        if mechanics.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(UiHostAppearanceProfileDenial::DuplicateMechanic);
        }
        let mut required = UiHostAppearanceMechanicFamily::ALL.to_vec();
        required.sort();
        if mechanics != required {
            return Err(UiHostAppearanceProfileDenial::MissingRequiredMechanic);
        }
        Ok(Self {
            posture: UiHostAppearanceProfilePosture::Current,
            identity,
            version,
            mechanics: mechanics.into_boxed_slice(),
            primary_pointer,
            geometry_qualification,
        })
    }

    pub const fn posture(&self) -> UiHostAppearanceProfilePosture {
        self.posture
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub const fn version(&self) -> u16 {
        self.version
    }
    pub fn mechanics(&self) -> &[UiHostAppearanceMechanicFamily] {
        &self.mechanics
    }
    pub const fn primary_pointer(&self) -> Option<super::UiHostPrimaryPointerKind> {
        self.primary_pointer
    }

    pub const fn geometry_qualification(&self) -> &UiHostAppearanceGeometryQualification {
        &self.geometry_qualification
    }

    pub(crate) fn append_canonical_encoding(&self, digest: &mut WorthUiHostCapabilityDigest) {
        digest.update_byte(match self.posture {
            UiHostAppearanceProfilePosture::Current => 1,
        });
        digest.update_text(self.identity.as_bytes());
        digest.update_u16(self.version);
        digest.update_u64(self.mechanics.len() as u64);
        for mechanic in &self.mechanics {
            digest.update_byte(mechanic.canonical_tag());
        }
        digest.update_byte(match self.primary_pointer {
            None => 0,
            Some(super::UiHostPrimaryPointerKind::Mouse) => 1,
            Some(super::UiHostPrimaryPointerKind::Pen) => 2,
        });
        self.geometry_qualification
            .append_canonical_encoding(digest);
    }
}

impl UiHostAppearanceMechanicFamily {
    const fn canonical_tag(self) -> u8 {
        match self {
            Self::SurfaceFill => 1,
            Self::SurfaceBorder => 2,
            Self::CornerRadii => 3,
            Self::Outline => 4,
            Self::TextRangeForeground => 5,
            Self::PortalSurface => 6,
            Self::Backdrop => 7,
            Self::OverlayOrder => 8,
            Self::PointerAffordance => 9,
            Self::Damage => 10,
            Self::Clip => 11,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        UiAppearanceLogicalLength, UiHostAppearanceGeometryQualificationBasis,
        UiHostAppearanceScaleGeometryQualification,
    };

    const EXPLICIT_MECHANICS: [UiHostAppearanceMechanicFamily; 11] = [
        UiHostAppearanceMechanicFamily::SurfaceFill,
        UiHostAppearanceMechanicFamily::SurfaceBorder,
        UiHostAppearanceMechanicFamily::CornerRadii,
        UiHostAppearanceMechanicFamily::Outline,
        UiHostAppearanceMechanicFamily::TextRangeForeground,
        UiHostAppearanceMechanicFamily::PortalSurface,
        UiHostAppearanceMechanicFamily::Backdrop,
        UiHostAppearanceMechanicFamily::OverlayOrder,
        UiHostAppearanceMechanicFamily::PointerAffordance,
        UiHostAppearanceMechanicFamily::Damage,
        UiHostAppearanceMechanicFamily::Clip,
    ];

    #[test]
    fn profile_requires_the_exhaustive_typed_mechanic_family() {
        assert!(UiHostAppearanceProfileContract::admit(
            "worth-ui-windows-dx12-v2",
            2,
            EXPLICIT_MECHANICS,
            Some(super::super::UiHostPrimaryPointerKind::Mouse),
            qualified_geometry(),
        )
        .is_ok());
        assert_eq!(
            UiHostAppearanceProfileContract::admit(
                "worth-ui-windows-dx12-v2",
                2,
                [UiHostAppearanceMechanicFamily::SurfaceFill],
                None,
                qualified_geometry(),
            ),
            Err(UiHostAppearanceProfileDenial::MissingRequiredMechanic)
        );
    }

    fn qualified_geometry() -> UiHostAppearanceGeometryQualification {
        UiHostAppearanceGeometryQualification::admit([
            UiHostAppearanceScaleGeometryQualification::new(
                1_000,
                1,
                UiAppearanceLogicalLength::new(1_000).unwrap(),
                UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
            ),
        ])
        .unwrap()
    }
}
