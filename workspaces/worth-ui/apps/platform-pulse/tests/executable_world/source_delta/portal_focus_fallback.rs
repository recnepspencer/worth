use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};

use super::atomic_replacement::{self, AppliedPulseSourceDelta, PulseSourceActionFailure};
use super::{PulseSourceDeltaDefinitionFailure, PulseSourceDeltaIdentity};

const PORTAL_PRIMARY_COMPONENT: &[u8] = b"component platform.pulse.component.portal_primary_target {\n  appearance { role platform.pulse.appearance.portal_primary_target }\n  interaction activate routes platform.pulse.action.route;\n}\n";

#[derive(Debug)]
pub(crate) struct PortalFocusFallbackSourceDelta {
    bytes: Box<[u8]>,
}

impl PortalFocusFallbackSourceDelta {
    pub(crate) fn from_checked_in(
        canonical: CanonicalPlatformPulse,
    ) -> Result<Self, PulseSourceDeltaDefinitionFailure> {
        let source = canonical.signals_source_bytes();
        let component = token_for_source_line_endings(source, PORTAL_PRIMARY_COMPONENT);
        let count = source
            .windows(component.len())
            .filter(|candidate| *candidate == component)
            .count();
        if count != 1 {
            return Err(match count {
                0 => PulseSourceDeltaDefinitionFailure::PortalPrimaryComponentMissing,
                count => PulseSourceDeltaDefinitionFailure::PortalPrimaryComponentAmbiguous(count),
            });
        }
        let offset = source
            .windows(component.len())
            .position(|row| row == component)
            .unwrap();
        let mut bytes = source[..offset].to_vec();
        bytes.extend_from_slice(&source[offset + component.len()..]);
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
        })
    }

    pub(crate) fn apply(
        self,
        installation: &IsolatedPulseInstallation,
    ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
        atomic_replacement::apply_path(
            installation.signals_source(),
            PulseSourceDeltaIdentity::PortalFocusFallback,
            &self.bytes,
        )
    }
}

fn token_for_source_line_endings(source: &[u8], token: &[u8]) -> Vec<u8> {
    if source
        .windows(token.len())
        .any(|candidate| candidate == token)
    {
        return token.to_vec();
    }
    let mut adapted = Vec::with_capacity(token.len());
    for byte in token {
        if *byte == b'\n' {
            adapted.push(b'\r');
        }
        adapted.push(*byte);
    }
    adapted
}
