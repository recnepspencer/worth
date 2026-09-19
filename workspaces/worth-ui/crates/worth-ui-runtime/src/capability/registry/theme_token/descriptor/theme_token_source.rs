#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeTokenSource {
    Platform,
    Application,
    PluginCustom,
    PluginAlias,
    PluginPlatformOverride,
}

impl ThemeTokenSource {
    pub fn platform() -> Self {
        Self::Platform
    }

    pub fn application() -> Self {
        Self::Application
    }

    pub fn plugin_custom() -> Self {
        Self::PluginCustom
    }

    pub fn plugin_alias() -> Self {
        Self::PluginAlias
    }

    pub fn plugin_platform_override_for_diagnostics() -> Self {
        Self::PluginPlatformOverride
    }

    pub(crate) fn claims_platform_override(&self) -> bool {
        matches!(self, Self::PluginPlatformOverride)
    }

    pub(crate) fn is_plugin_contribution(&self) -> bool {
        matches!(
            self,
            Self::PluginCustom | Self::PluginAlias | Self::PluginPlatformOverride
        )
    }

    pub(crate) fn digest_basis(&self) -> &'static str {
        match self {
            Self::Platform => "platform",
            Self::Application => "application",
            Self::PluginCustom => "plugin_custom",
            Self::PluginAlias => "plugin_alias",
            Self::PluginPlatformOverride => "plugin_platform_override",
        }
    }
}
