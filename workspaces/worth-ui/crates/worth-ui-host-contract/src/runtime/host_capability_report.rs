use crate::UiHostAppearanceProfileContract;

use super::{WorthUiHostCapability, WorthUiHostCapabilityPosture};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiHostCapabilityObservationGeneration {
    value: u64,
}

impl WorthUiHostCapabilityObservationGeneration {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    pub const fn as_u64(self) -> u64 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiHostCapabilityReport {
    observation_generation: WorthUiHostCapabilityObservationGeneration,
    posture: WorthUiHostCapabilityPosture,
    observed_capabilities: Box<[WorthUiHostCapability]>,
    appearance_profile: Option<UiHostAppearanceProfileContract>,
}

impl WorthUiHostCapabilityReport {
    pub fn available(capabilities: Vec<WorthUiHostCapability>) -> Self {
        Self::new(WorthUiHostCapabilityPosture::Available, capabilities)
    }

    pub fn missing(capabilities: Vec<WorthUiHostCapability>) -> Self {
        Self::new(WorthUiHostCapabilityPosture::Missing, capabilities)
    }

    pub fn ambiguous(capabilities: Vec<WorthUiHostCapability>) -> Self {
        Self::new(WorthUiHostCapabilityPosture::Ambiguous, capabilities)
    }

    pub fn diagnostic_only(capabilities: Vec<WorthUiHostCapability>) -> Self {
        Self::new(WorthUiHostCapabilityPosture::DiagnosticOnly, capabilities)
    }

    fn new(
        posture: WorthUiHostCapabilityPosture,
        mut observed_capabilities: Vec<WorthUiHostCapability>,
    ) -> Self {
        observed_capabilities.sort_by_key(|capability| capability.as_str());
        observed_capabilities.dedup();

        Self {
            observation_generation: WorthUiHostCapabilityObservationGeneration::new(0),
            posture,
            observed_capabilities: observed_capabilities.into_boxed_slice(),
            appearance_profile: None,
        }
    }

    /// Attach the host-owned appearance qualification profile to this report.
    ///
    /// The profile contract is explicitly staged and non-current. Attaching it
    /// records qualification support for certification consumers; it does not
    /// change the live host protocol or enable appearance emission.
    pub fn with_appearance_profile(
        mut self,
        appearance_profile: UiHostAppearanceProfileContract,
    ) -> Self {
        self.appearance_profile = Some(appearance_profile);
        self
    }

    pub fn with_observation_generation(
        mut self,
        observation_generation: WorthUiHostCapabilityObservationGeneration,
    ) -> Self {
        self.observation_generation = observation_generation;
        self
    }

    pub fn observation_generation(&self) -> WorthUiHostCapabilityObservationGeneration {
        self.observation_generation
    }

    pub fn posture(&self) -> WorthUiHostCapabilityPosture {
        self.posture
    }

    pub fn observed_capabilities(&self) -> &[WorthUiHostCapability] {
        &self.observed_capabilities
    }

    pub fn appearance_profile(&self) -> Option<&UiHostAppearanceProfileContract> {
        self.appearance_profile.as_ref()
    }

    pub fn profile_identity_digest(&self) -> u64 {
        let mut digest = WorthUiHostCapabilityDigest::new();
        digest.update_byte(capability_posture_tag(self.posture));
        digest.update_u64(self.observed_capabilities.len() as u64);
        for capability in &self.observed_capabilities {
            digest.update_text(capability.as_str().as_bytes());
        }
        match &self.appearance_profile {
            None => digest.update_byte(0),
            Some(profile) => {
                digest.update_byte(1);
                profile.append_canonical_encoding(&mut digest);
            }
        }
        digest.finish()
    }

    pub fn supports(&self, capability: WorthUiHostCapability) -> bool {
        self.observed_capabilities.contains(&capability)
    }
}

fn capability_posture_tag(posture: WorthUiHostCapabilityPosture) -> u8 {
    match posture {
        WorthUiHostCapabilityPosture::Available => 1,
        WorthUiHostCapabilityPosture::Missing => 2,
        WorthUiHostCapabilityPosture::Ambiguous => 3,
        WorthUiHostCapabilityPosture::DiagnosticOnly => 4,
    }
}

pub(crate) struct WorthUiHostCapabilityDigest {
    value: u64,
}

impl WorthUiHostCapabilityDigest {
    pub(crate) fn new() -> Self {
        let mut digest = Self {
            value: 0xCBF2_9CE4_8422_2325,
        };
        digest.update_bytes(b"worth-ui-host-capability-report\0");
        digest
    }

    pub(crate) fn update_byte(&mut self, byte: u8) {
        self.value = self.value.wrapping_mul(0x0000_0100_0000_01B3) ^ u64::from(byte);
    }

    pub(crate) fn update_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.update_byte(*byte);
        }
    }

    pub(crate) fn update_u16(&mut self, value: u16) {
        self.update_bytes(&value.to_le_bytes());
    }

    pub(crate) fn update_u64(&mut self, value: u64) {
        self.update_bytes(&value.to_le_bytes());
    }

    pub(crate) fn update_text(&mut self, text: &[u8]) {
        self.update_u64(text.len() as u64);
        self.update_bytes(text);
    }

    pub(crate) fn finish(self) -> u64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{WorthUiHostCapability, WorthUiHostCapabilityReport};
    use crate::{
        UiAppearanceLogicalLength, UiHostAppearanceGeometryQualification,
        UiHostAppearanceGeometryQualificationBasis, UiHostAppearanceMechanicFamily,
        UiHostAppearanceProfileContract, UiHostAppearanceProfilePosture,
        UiHostAppearanceScaleGeometryQualification, UiHostPrimaryPointerKind,
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
    fn profile_digest_uses_constructor_canonical_order_without_input_order_leakage() {
        let first = WorthUiHostCapabilityReport::available(vec![
            WorthUiHostCapability::Ime,
            WorthUiHostCapability::Accessibility,
            WorthUiHostCapability::Ime,
        ]);
        let second = WorthUiHostCapabilityReport::available(vec![
            WorthUiHostCapability::Accessibility,
            WorthUiHostCapability::Ime,
        ]);

        assert_eq!(first, second);
        assert_eq!(
            first.profile_identity_digest(),
            second.profile_identity_digest()
        );
    }

    #[test]
    fn appearance_profile_digest_is_canonical_and_absence_is_distinct() {
        let first_profile = appearance_profile(
            "worth-ui-windows-dx12-v2",
            2,
            EXPLICIT_MECHANICS,
            Some(UiHostPrimaryPointerKind::Mouse),
        );
        let mut reversed_mechanics = EXPLICIT_MECHANICS;
        reversed_mechanics.reverse();
        let second_profile = appearance_profile(
            "worth-ui-windows-dx12-v2",
            2,
            reversed_mechanics,
            Some(UiHostPrimaryPointerKind::Mouse),
        );
        let absent = WorthUiHostCapabilityReport::available(vec![]);
        let first = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(first_profile.clone());
        let second =
            WorthUiHostCapabilityReport::available(vec![]).with_appearance_profile(second_profile);

        assert_eq!(first, second);
        assert_eq!(
            first.profile_identity_digest(),
            second.profile_identity_digest()
        );
        assert_eq!(
            first.appearance_profile(),
            Some(&first_profile),
            "the report reader exposes the current profile without reconstructing it"
        );
        assert_eq!(
            first_profile.posture(),
            UiHostAppearanceProfilePosture::Current
        );
        assert_ne!(
            absent.profile_identity_digest(),
            first.profile_identity_digest()
        );
        assert!(absent.appearance_profile().is_none());
    }

    #[test]
    fn appearance_profile_digest_carries_identity_version_mechanics_and_pointer_posture() {
        let baseline = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(appearance_profile(
                "worth-ui-windows-dx12-v2",
                2,
                EXPLICIT_MECHANICS,
                Some(UiHostPrimaryPointerKind::Mouse),
            ))
            .profile_identity_digest();
        let different_identity = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(appearance_profile(
                "worth-ui-windows-dx12-v3",
                2,
                EXPLICIT_MECHANICS,
                Some(UiHostPrimaryPointerKind::Mouse),
            ))
            .profile_identity_digest();
        let different_version = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(appearance_profile(
                "worth-ui-windows-dx12-v2",
                3,
                EXPLICIT_MECHANICS,
                Some(UiHostPrimaryPointerKind::Mouse),
            ))
            .profile_identity_digest();
        let no_pointer = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(appearance_profile(
                "worth-ui-windows-dx12-v2",
                2,
                EXPLICIT_MECHANICS,
                None,
            ))
            .profile_identity_digest();
        let pen_pointer = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(appearance_profile(
                "worth-ui-windows-dx12-v2",
                2,
                EXPLICIT_MECHANICS,
                Some(UiHostPrimaryPointerKind::Pen),
            ))
            .profile_identity_digest();
        let different_geometry = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(
                UiHostAppearanceProfileContract::admit(
                    "worth-ui-windows-dx12-v2",
                    2,
                    EXPLICIT_MECHANICS,
                    Some(UiHostPrimaryPointerKind::Mouse),
                    geometry(2_000, 1, 500),
                )
                .unwrap(),
            )
            .profile_identity_digest();
        let different_fringe = WorthUiHostCapabilityReport::available(vec![])
            .with_appearance_profile(
                UiHostAppearanceProfileContract::admit(
                    "worth-ui-windows-dx12-v2",
                    2,
                    EXPLICIT_MECHANICS,
                    Some(UiHostPrimaryPointerKind::Mouse),
                    geometry(1_000, 2, 2_000),
                )
                .unwrap(),
            )
            .profile_identity_digest();

        assert_ne!(baseline, different_identity);
        assert_ne!(baseline, different_version);
        assert_ne!(baseline, no_pointer);
        assert_ne!(baseline, pen_pointer);
        assert_ne!(no_pointer, pen_pointer);
        assert_ne!(baseline, different_geometry);
        assert_ne!(baseline, different_fringe);
    }

    fn appearance_profile(
        identity: &str,
        version: u16,
        mechanics: impl IntoIterator<Item = UiHostAppearanceMechanicFamily>,
        pointer: Option<UiHostPrimaryPointerKind>,
    ) -> UiHostAppearanceProfileContract {
        UiHostAppearanceProfileContract::admit(
            identity,
            version,
            mechanics,
            pointer,
            geometry(1_000, 1, 1_000),
        )
        .expect("the explicit complete appearance mechanic list must admit")
    }

    fn geometry(
        scale: u32,
        physical_pixels: u32,
        logical_subpixels: u32,
    ) -> UiHostAppearanceGeometryQualification {
        UiHostAppearanceGeometryQualification::admit([
            UiHostAppearanceScaleGeometryQualification::new(
                scale,
                physical_pixels,
                UiAppearanceLogicalLength::new(logical_subpixels as i32).unwrap(),
                UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
            ),
        ])
        .unwrap()
    }
}
