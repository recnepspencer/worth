#[derive(Default)]
pub(crate) struct ThemeRegistry {
    bundle: Option<super::FrozenAppearanceThemeCapabilities>,
}

impl ThemeRegistry {
    pub(crate) const REGISTRATION_IDENTITY: &'static str = "appearance-theme-bundle";
    pub(crate) fn install(
        &mut self,
        bundle: super::FrozenAppearanceThemeCapabilities,
    ) -> Result<
        crate::capability::RegistrationCandidate,
        super::FrozenAppearanceThemeCapabilitiesDenial,
    > {
        if self.bundle.is_some() {
            return Err(super::FrozenAppearanceThemeCapabilitiesDenial::BundleAlreadyInstalled);
        }
        self.bundle = Some(bundle);
        Ok(crate::capability::RegistrationCandidate::new(
            crate::capability::APPEARANCE_THEME_FAMILY_NAME,
            Self::REGISTRATION_IDENTITY,
            crate::capability::CapabilitySupportKind::Admitted,
        ))
    }

    pub(crate) fn freeze(
        self,
        accepted: &super::AppearanceThemeAcceptedRegistrationProof,
    ) -> Option<super::FrozenAppearanceThemeCapabilities> {
        self.bundle.filter(|_| accepted.admits_bundle())
    }
}
