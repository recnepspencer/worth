#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativePlatformProfileIdentity(&'static str);

impl UiNativePlatformProfileIdentity {
    pub const WORTH_UI_WINDOWS_DX12_V2: Self = Self("worth-ui-windows-dx12-v2");
    pub const WORTH_UI_LINUX_WAYLAND_VULKAN_V1: Self = Self("worth-ui-linux-wayland-vulkan-v1");
    pub const WORTH_UI_LINUX_X11_VULKAN_V1: Self = Self("worth-ui-linux-x11-vulkan-v1");
    /// The X11 certification profile: identical to the hardware X11 profile
    /// except that it admits a software-rasterizing adapter, because an X
    /// server without DRI3 (Xvfb) offers hardware adapters no surface formats.
    pub const WORTH_UI_LINUX_X11_VULKAN_SOFTWARE_V1: Self =
        Self("worth-ui-linux-x11-vulkan-software-v1");

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
