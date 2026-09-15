use serde::{Deserialize, Serialize};

use worth_foundational::ObservationActivationProfile;
use worth_signal::facade::RuntimePolicy as NativeRuntimePolicy;

use crate::boundary::errors::WorthSignalJsError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePolicySpec {
    pub preset: RuntimePolicyPreset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimePolicyPreset {
    Development,
    Operational,
    Forensic,
    WebDevelopment,
    Fintech,
    Kernel,
    GameEngine,
}

impl Default for RuntimePolicySpec {
    fn default() -> Self {
        Self {
            preset: RuntimePolicyPreset::WebDevelopment,
        }
    }
}

impl RuntimePolicySpec {
    pub fn into_native(self) -> Result<NativeRuntimePolicy, WorthSignalJsError> {
        let policy = match self.preset {
            RuntimePolicyPreset::Development => NativeRuntimePolicy::development(),
            RuntimePolicyPreset::Operational => NativeRuntimePolicy::operational(),
            RuntimePolicyPreset::Forensic => NativeRuntimePolicy::forensic(),
            // The wasm product contract promises after-the-fact diagnostics
            // (`diagnostics.latestFlow()` / `latestObservation()` return the
            // most recent complete flow, including `change.changed_aspects`)
            // without any observation-session ceremony, and the JS surface has
            // no API to open one. The native `web_development()` preset became
            // on-demand (surface mask 0) in Milestone 10, which silently turned
            // every wasm flow summary into `null`. The default web preset
            // therefore keeps observation continuous; `Operational`, `Kernel`
            // and `GameEngine` stay on-demand for callers who opt in to them.
            RuntimePolicyPreset::WebDevelopment => NativeRuntimePolicy::web_development()
                .with_observation_activation(ObservationActivationProfile::Continuous),
            RuntimePolicyPreset::Fintech => NativeRuntimePolicy::fintech(),
            RuntimePolicyPreset::Kernel => NativeRuntimePolicy::kernel(),
            RuntimePolicyPreset::GameEngine => NativeRuntimePolicy::game_engine(),
        };
        Ok(policy)
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::ObservationActivationProfile;

    use super::{RuntimePolicyPreset, RuntimePolicySpec};

    #[test]
    fn default_web_development_preset_keeps_observation_continuous() {
        let native = RuntimePolicySpec::default().into_native().unwrap();
        assert_eq!(
            native.observation_activation(),
            ObservationActivationProfile::Continuous,
            "the wasm product promises `latestFlow()` without an observation session",
        );
    }

    #[test]
    fn explicit_operational_preset_stays_on_demand() {
        let native = RuntimePolicySpec {
            preset: RuntimePolicyPreset::Operational,
        }
        .into_native()
        .unwrap();
        assert_eq!(
            native.observation_activation(),
            ObservationActivationProfile::OnDemand,
        );
    }
}
