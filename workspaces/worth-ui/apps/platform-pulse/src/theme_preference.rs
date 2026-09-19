use serde::Deserialize;
#[cfg(test)]
mod tests;
mod watch;
pub(crate) use watch::PlatformPulseThemePreferenceWatch;
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlatformPulseThemeChoice {
    Default,
    Alternate,
}

impl PlatformPulseThemeChoice {
    pub(crate) fn definition(self) -> worth_ui::facade::appearance::UiThemeDefinitionIdentity {
        worth_ui::facade::appearance::UiThemeDefinitionIdentity::new(match self {
            Self::Default => "theme.platform_pulse.default",
            Self::Alternate => "theme.platform_pulse.alternate",
        })
        .expect("declared Pulse theme identity")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlatformPulseThemePreference {
    pub(crate) schema_version: u16,
    pub(crate) revision: u64,
    pub(crate) theme: PlatformPulseThemeChoice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseThemePreferenceDenial {
    WatchUnavailable,
    WorkerPanicked,
    ReadUnavailable,
    Oversized,
    InvalidRecord,
    UnsupportedVersion,
    StaleRevision,
}
