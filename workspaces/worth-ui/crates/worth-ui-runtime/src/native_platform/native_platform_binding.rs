/// Move-only proof that one prepared platform owns one application slot.
/// Construction and consumption remain inside native-platform composition.
pub(crate) struct UiNativePlatformBindingGrant {
    profile: worth_ui_host_native::UiNativePlatformProfileIdentity,
    preparation_identity: u64,
}

impl UiNativePlatformBindingGrant {
    pub(super) const fn issue(preparation_identity: u64) -> Self {
        Self {
            profile:
                worth_ui_host_native::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2,
            preparation_identity,
        }
    }

    pub(super) const fn preparation_identity(&self) -> u64 {
        self.preparation_identity
    }

    pub(super) const fn profile(&self) -> worth_ui_host_native::UiNativePlatformProfileIdentity {
        self.profile
    }
}
